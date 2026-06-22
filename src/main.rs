use std::path::Path;
use anyhow::{bail, Context, Result};
use askama::Template;

mod content;
mod data;
mod output;
mod render;
mod spellblock;
mod validator;

fn main() -> Result<()> {
    let content_root: &Path = Path::new("content");
    let static_root: &Path = Path::new("static");
    let data_root: &Path = Path::new("data");
    let output_root: &Path = Path::new("dist");

    // Start every build from an empty dist/ so deleted content doesn't linger.
    if output_root.exists() {
        std::fs::remove_dir_all(output_root)
            .with_context(|| format!("clearing {}", output_root.display()))?;
    }

    output::copy_static(static_root, output_root)
        .context("copying static assets")?;

    let now = data::load_now(&data_root.join("now.toml"))
        .context("loading data/now.toml")?
        .active();  // sidebar only shows entries without ended_at
    let connect = data::load_connect(&data_root.join("connect.toml"))
        .context("loading data/connect.toml")?;
    let forges_flat = data::load_forges(&data_root.join("forges.toml"))
        .context("loading data/forges.toml")?;
    let forges = render::group_forges(forges_flat);

    // The public origin the build prints into canonical tags and the Atom
    // feed (Issues 2 & 4). Loaded and echoed now so a missing or malformed
    // data/site.toml fails the build loudly, before anything depends on it.
    let site = data::load_site(&data_root.join("site.toml"))
        .context("loading data/site.toml")?;
    println!("site base: {}", site.base_url);

    // "Own Your Stack" identity - masthead strings, reused by every newsletter
    // issue page and (Issues 3 & 4) the /newsletter/ index and Atom feed.
    let newsletter = data::load_newsletter(&data_root.join("newsletter.toml"))
        .context("loading data/newsletter.toml")?;

    let md_files = content::discover(content_root)
        .context("discovering content")?;

    // Phase 1: parse every file
    let mut documents = Vec::new();
    for path in &md_files {
        let doc = content::parse(path)
            .with_context(|| format!("parsing {}", path.display()))?;
        documents.push((path.clone(), doc));
    }

    // Phase 2: validate every file's frontmatter (drafts included)
    let mut all_errors = Vec::new();
    for (path, doc) in &documents {
        all_errors.extend(validator::validate(&doc.frontmatter, path));
    }

    if !all_errors.is_empty() {
        eprintln!("\n{} frontmatter validation error(s):\n", all_errors.len());
        for err in &all_errors {
            eprintln!("  - {}", err);
        }
        eprintln!();
        bail!("frontmatter validation failed");
    }

    // Phase 2.5: filter drafts out of the build
    let total = documents.len();
    documents.retain(|(_, doc)| !doc.frontmatter.draft);
    let skipped = total - documents.len();
    if skipped > 0 {
        eprintln!("skipped {} draft(s)", skipped);
    }

    // Phase 3: prepare every non-draft file (render body + build the Article),
    // but defer writing - the "related posts" block needs every post's tags,
    // which aren't all known until this loop finishes.
    let mut prepared = Vec::new();
    for (path, doc) in documents {
        let p = prepare_article(&path, doc, content_root, &site)
            .with_context(|| format!("rendering {}", path.display()))?;
        prepared.push(p);
    }

    // All post metadata, cloned out for the cross-post features below
    // (related, quotes, newsletter list, sitemap, index partition).
    let posts: Vec<render::Post> = prepared.iter().map(|p| p.post.clone()).collect();
    let post_count = posts.len();

    // Phase 3.5: now that every post is known, compute each article's related
    // posts (by shared tags) and write the final HTML.
    for prep in &mut prepared {
        prep.article.related = related_posts(&prep.post, &posts);
        let page_html = prep.article.render().context("rendering article template")?;
        let written = output::write_html(content_root, output_root, &prep.content_path, &page_html)?;
        println!("wrote {} ({})", written.display(), prep.article.title);
    }

    // Collect pull-quotes from every post that has one, for the index rotation
    // panel. Philosophy is personal writing and never feeds the rotation, even
    // if an essay carries a `quote`.
    let quotes: Vec<render::Quote> = posts.iter()
        .filter(|p| p.kind != "philosophy")
        .filter_map(|p| {
            p.quote.as_ref().map(|q| render::Quote {
                text: q.clone(),
                post_title: p.title.clone(),
                post_url: p.url.clone(),
                post_date: p.published_at_display.clone(),
            })
        })
        .collect();
    let quotes_json = serde_json::to_string(&quotes)
        .context("serializing quotes for index")?;

    // Newsletter issues, newest first - the /newsletter/ landing page lists
    // these. Taken before the partition below consumes `posts`.
    let mut newsletter_issues: Vec<render::Post> = posts
        .iter()
        .filter(|p| p.kind == "newsletter")
        .cloned()
        .collect();
    newsletter_issues.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    // Philosophy essays, newest first - the /philosophy/ landing page lists these.
    // Personal writing, walled into its own section but still left in the home feed.
    let mut philosophy_essays: Vec<render::Post> = posts
        .iter()
        .filter(|p| p.kind == "philosophy")
        .cloned()
        .collect();
    philosophy_essays.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    // Sitemap entries - every page. Built before the partition consumes `posts`.
    // The home page's lastmod tracks the newest post anywhere; the section indexes
    // track their newest entry. Both give crawlers a real freshness signal.
    let newest_post_date = posts.iter().map(|p| &p.published_at).max().cloned();
    let newest_issue_date = newsletter_issues.first().map(|p| p.published_at.clone());
    let newest_philosophy_date = philosophy_essays.first().map(|p| p.published_at.clone());
    let mut sitemap_urls = vec![
        render::SitemapUrl { loc: format!("{}/", site.base_url), lastmod: newest_post_date },
        render::SitemapUrl { loc: format!("{}/newsletter/", site.base_url), lastmod: newest_issue_date },
        render::SitemapUrl { loc: format!("{}/philosophy/", site.base_url), lastmod: newest_philosophy_date },
    ];
    for p in &posts {
        sitemap_urls.push(render::SitemapUrl {
            loc: format!("{}{}", site.base_url, p.url),
            lastmod: Some(p.published_at.clone()),
        });
    }

    // Partition into featured + recent. Both sorted most-recent first.
    let (mut featured_all, mut recent_posts): (Vec<_>, Vec<_>) = posts
        .into_iter()
        .partition(|p| p.featured);
    featured_all.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    recent_posts.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    // Exactly one post may wear `featured: true`. Two would stack in the FEATURED
    // slot and bury the lede. Merc the build loudly, naming the newest (keep it)
    // and every older one to un-feature. featured_all is already newest-first.
    if featured_all.len() > 1 {
        let newest = &featured_all[0];
        eprintln!(
            "\n{} posts are marked `featured: true`, but only one is allowed.\n",
            featured_all.len(),
        );
        eprintln!("  KEEP (newest):  \"{}\" ({})", newest.title, newest.published_at);
        eprintln!("  Remove `featured: true` from the older one(s):");
        for older in &featured_all[1..] {
            eprintln!("    - \"{}\" ({})", older.title, older.published_at);
        }
        eprintln!();
        bail!("more than one featured post");
    }
    let featured_posts: Vec<_> = featured_all;

    let index = render::Index {
        featured_posts,
        posts: recent_posts,
        now,
        connect,
        forges,
        quotes_json,
        meta: render::PageMeta {
            canonical_url: format!("{}/", site.base_url),
            body_class: String::new(),
            og_title: "Mathew Storm - Personal Site".to_string(),
            og_description: site.description.clone(),
            og_type: "website".to_string(),
            og_image: og_image_url(&site),
            og_image_alt: if site.og_image.is_empty() { String::new() } else { "Mathew Storm".to_string() },
            published_time: String::new(),
            json_ld: site_json_ld(&site),
        },
    };
    let index_html = index.render().context("rendering index")?;
    output::write_index(output_root, &index_html)?;
    println!("wrote {}/index.html ({} posts)", output_root.display(), post_count);

    // The /newsletter/ landing page - "Own Your Stack" masthead, pitch, and
    // the issue list. Resolves the /newsletter/ link advertised in now.toml.
    let newsletter_page = render::NewsletterIndex {
        meta: render::PageMeta {
            canonical_url: format!("{}/newsletter/", site.base_url),
            body_class: "oys".to_string(),
            og_title: newsletter.name.clone(),
            og_description: newsletter.tagline.clone(),
            og_type: "website".to_string(),
            og_image: og_image_url(&site),
            og_image_alt: if site.og_image.is_empty() { String::new() } else { newsletter.name.clone() },
            published_time: String::new(),
            json_ld: site_json_ld(&site),
        },
        name: newsletter.name.clone(),
        tagline: newsletter.tagline.clone(),
        issues: newsletter_issues.clone(),
    };
    let newsletter_html = newsletter_page.render().context("rendering newsletter index")?;
    let written = output::write_at(output_root, "newsletter/index.html", &newsletter_html)?;
    println!("wrote {} ({} issue(s))", written.display(), newsletter_page.issues.len());

    // The /philosophy/ landing page - the personal-writing disclaimer and the
    // essay list. Renders even when empty, so the section URL always resolves.
    let philosophy_page = render::PhilosophyIndex {
        meta: render::PageMeta {
            canonical_url: format!("{}/philosophy/", site.base_url),
            body_class: "philo".to_string(),
            og_title: "Philosophy - mathewstorm.ca".to_string(),
            og_description: "Personal philosophical writing - separate from my engineering work, though it informs how and why I build.".to_string(),
            og_type: "website".to_string(),
            og_image: og_image_url(&site),
            og_image_alt: if site.og_image.is_empty() { String::new() } else { "Mathew Storm".to_string() },
            published_time: String::new(),
            json_ld: philosophy_collection_json_ld(&site, &philosophy_essays),
        },
        essays: philosophy_essays.clone(),
    };
    let philosophy_html = philosophy_page.render().context("rendering philosophy index")?;
    let written = output::write_at(output_root, "philosophy/index.html", &philosophy_html)?;
    println!("wrote {} ({} essay(s))", written.display(), philosophy_page.essays.len());

    // The Atom feed - newsletter issues only. Absolute URLs and RFC3339
    // timestamps; published_at is YYYY-MM-DD, so midnight UTC is appended.
    let feed_updated = newsletter_issues
        .first()
        .map(|p| format!("{}T00:00:00Z", p.published_at))
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());
    let feed = render::Feed {
        base_url: site.base_url.clone(),
        feed_url: format!("{}/newsletter/feed.xml", site.base_url),
        home_url: format!("{}/newsletter/", site.base_url),
        title: newsletter.name.clone(),
        subtitle: newsletter.tagline.clone(),
        updated: feed_updated,
        entries: newsletter_issues,
    };
    let feed_xml = feed.render().context("rendering newsletter feed")?;
    let written = output::write_at(output_root, "newsletter/feed.xml", &feed_xml)?;
    println!("wrote {} ({} entries)", written.display(), feed.entries.len());

    // sitemap.xml - every page, for search-engine discovery.
    let sitemap = render::Sitemap { urls: sitemap_urls };
    let sitemap_xml = sitemap.render().context("rendering sitemap")?;
    let written = output::write_at(output_root, "sitemap.xml", &sitemap_xml)?;
    println!("wrote {} ({} urls)", written.display(), sitemap.urls.len());

    Ok(())
}

/// An article rendered far enough to know its own metadata, but not yet written
/// to disk - it still needs its `related` block, which depends on every other
/// post. Carries the output path so the write can happen in a later pass.
struct Prepared {
    content_path: std::path::PathBuf,
    article: render::Article,
    post: render::Post,
}

fn prepare_article(
    content_path: &Path,
    doc: content::Document,
    content_root: &Path,
    site: &data::Site,
) -> Result<Prepared> {
    let body_html = spellblock::render(&doc.body)
        .context("expanding SpellBlocks")?;

    let title = doc.frontmatter.title.clone();
    let description = doc.frontmatter.description.clone();
    let url = output::url_for(content_root, content_path)?;
    let published_at = doc.frontmatter.published_at.clone();
    let published_at_display = format_date(&published_at);
    let tags = doc.frontmatter.tags.clone();
    let kind = doc.frontmatter.kind.clone();
    let kind_display = kind.to_uppercase();
    let quote = doc.frontmatter.quote.clone();
    let reading_time = reading_time_from_html(&body_html);
    let featured = doc.frontmatter.featured;

    let is_newsletter = kind == "newsletter";
    let issue_display = doc.frontmatter.issue.map(|n| format!("{n:03}"));

    // A newsletter issue wears the "Own Your Stack" identity: the .oys body
    // scope, the masthead, an issue number, and a breadcrumb back to the
    // newsletter home. Every other post keeps the plain article treatment.
    let body_class = if is_newsletter { "oys".to_string() } else { String::new() };
    let (breadcrumb_href, breadcrumb_label) = match kind.as_str() {
        "newsletter" => ("/newsletter/".to_string(), "all issues".to_string()),
        "philosophy" => ("/philosophy/".to_string(), "all essays".to_string()),
        _ => ("/".to_string(), "all writing".to_string()),
    };
    let newsletter = is_newsletter.then(|| render::NewsletterIssue {
        // The validator guarantees a newsletter issue carries `issue`.
        issue_display: issue_display.clone().unwrap_or_default(),
    });

    // Per-post social card: the post's own `cover` wins; otherwise fall back to
    // the site-wide og_image. A missing cover is fine - og:image just omits.
    let canonical_url = format!("{}{}", site.base_url, url);
    let cover = doc.frontmatter.cover.clone().filter(|s| !s.trim().is_empty());
    let cover_alt = doc.frontmatter.cover_alt.clone().unwrap_or_else(|| title.clone());
    let og_image = match &cover {
        Some(path) => format!("{}{}", site.base_url, path),
        None => og_image_url(site),
    };
    let og_image_alt = if og_image.is_empty() { String::new() } else { cover_alt.clone() };
    let og_description = description.clone().unwrap_or_else(|| site.description.clone());

    // Entities this post is `about`: every registered entity whose tag the post
    // carries. Drives the schema.org `about` on the BlogPosting.
    let about: Vec<data::Entity> = site.entities.iter()
        .filter(|e| tags.contains(&e.tag))
        .cloned()
        .collect();

    let json_ld = article_json_ld(
        site, &title, &og_description, &canonical_url, &published_at,
        cover.as_deref(), &breadcrumb_href, &breadcrumb_label, &about,
    );

    let meta = render::PageMeta {
        canonical_url,
        body_class,
        og_title: title.clone(),
        og_description,
        og_type: "article".to_string(),
        og_image,
        og_image_alt,
        // YYYY-MM-DD is a valid ISO 8601 date; good enough for article:published_time.
        published_time: published_at.clone(),
        json_ld,
    };

    let article = render::Article {
        title: doc.frontmatter.title,
        kind: kind.clone(),
        tags: doc.frontmatter.tags,
        body: body_html.clone(),
        published_at_display: published_at_display.clone(),
        project: doc.frontmatter.project,
        project_url: doc.frontmatter.project_url,
        pr_url: doc.frontmatter.pr_url,
        meta,
        breadcrumb_href,
        breadcrumb_label,
        newsletter,
        cover: cover.clone(),
        cover_alt,
        related: Vec::new(), // filled in a later pass, once all posts are known
    };

    let post = render::Post {
        title,
        description,
        url,
        tags,
        published_at,
        published_at_display,
        kind,
        kind_display,
        quote,
        reading_time,
        featured,
        issue_display,
        cover,
        body: body_html,
    };

    Ok(Prepared { content_path: content_path.to_path_buf(), article, post })
}

/// Up to three sibling posts sharing the most tags with `post`, most-shared
/// first, then most recent. Drives both the on-page "Related" block and the
/// internal-linking that builds topical clusters for SEO.
fn related_posts(post: &render::Post, all: &[render::Post]) -> Vec<render::RelatedPost> {
    let mut scored: Vec<(usize, &render::Post)> = all
        .iter()
        .filter(|c| c.url != post.url)
        .filter_map(|c| {
            let shared = c.tags.iter().filter(|t| post.tags.contains(t)).count();
            (shared > 0).then_some((shared, c))
        })
        .collect();
    // Most shared tags first; ties broken by most recent.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.published_at.cmp(&a.1.published_at)));
    scored
        .into_iter()
        .take(3)
        .map(|(_, c)| render::RelatedPost {
            title: c.title.clone(),
            url: c.url.clone(),
            kind_display: c.kind_display.clone(),
            published_at_display: c.published_at_display.clone(),
        })
        .collect()
}

fn reading_time_from_html(html: &str) -> u32 {
    // Strip HTML tags by scanning character-by-character. Cheap and good enough
    // for word counting - pulldown-cmark output is well-formed.
    let mut in_tag = false;
    let mut text = String::with_capacity(html.len());
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let words = text.split_whitespace().count();
    let minutes = ((words as f64) / 200.0).ceil() as u32;
    minutes.max(1)
}

/// Absolute URL of the social-share image, or "" when none is configured.
fn og_image_url(site: &data::Site) -> String {
    if site.og_image.is_empty() {
        String::new()
    } else {
        format!("{}{}", site.base_url, site.og_image)
    }
}

/// Serialize a JSON-LD value for safe embedding inside a <script> tag. serde_json
/// produces valid JSON; we additionally escape `<`, `>`, `&` so a stray
/// "</script>" or entity in any string can never break out of the element.
fn to_script_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

/// The site-stable schema.org graph: the WebSite plus its author Person. Emitted
/// on the home page and the newsletter index so Google can resolve the site to a
/// single entity (the path to a Knowledge Panel). Returns "" if no person.name.
fn site_json_ld(site: &data::Site) -> String {
    if site.person.name.trim().is_empty() {
        return String::new();
    }
    let person_id = format!("{}/#person", site.base_url);
    let graph = serde_json::json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "WebSite",
                "@id": format!("{}/#website", site.base_url),
                "url": format!("{}/", site.base_url),
                "name": "mathewstorm.ca",
                "description": site.description,
                "publisher": { "@id": person_id },
                "inLanguage": "en",
            },
            person_node(site, &person_id),
        ],
    });
    to_script_json(&graph)
}

/// The /philosophy/ index as a schema.org CollectionPage whose `hasPart` lists
/// every essay (each a BlogPosting authored by Mathew). This hands Google a
/// named topical cluster - "the philosophy Mathew Storm wrote" - in one node.
fn philosophy_collection_json_ld(site: &data::Site, essays: &[render::Post]) -> String {
    if site.person.name.trim().is_empty() {
        return String::new();
    }
    let person_id = format!("{}/#person", site.base_url);
    let url = format!("{}/philosophy/", site.base_url);
    let parts: Vec<serde_json::Value> = essays.iter().map(|p| {
        serde_json::json!({
            "@type": "BlogPosting",
            "headline": p.title,
            "url": format!("{}{}", site.base_url, p.url),
            "datePublished": p.published_at,
            "author": { "@id": person_id },
        })
    }).collect();
    let collection = serde_json::json!({
        "@type": "CollectionPage",
        "@id": url,
        "url": url,
        "name": "Philosophy",
        "isPartOf": { "@id": format!("{}/#website", site.base_url) },
        "author": { "@id": person_id },
        "hasPart": parts,
    });
    let graph = serde_json::json!({
        "@context": "https://schema.org",
        "@graph": [ collection, person_node(site, &person_id) ],
    });
    to_script_json(&graph)
}

/// The schema.org Person node, shared by the site graph and every article's
/// author/publisher. `job_title` and `same_as` are included only when present.
fn person_node(site: &data::Site, person_id: &str) -> serde_json::Value {
    let mut node = serde_json::json!({
        "@type": "Person",
        "@id": person_id,
        "name": site.person.name,
        "url": format!("{}/", site.base_url),
    });
    let map = node.as_object_mut().expect("person node is an object");
    if !site.person.job_title.trim().is_empty() {
        map.insert("jobTitle".into(), site.person.job_title.clone().into());
    }
    // hasOccupation is the entity-level claim ("Philosopher") that Google reads
    // for the Knowledge Graph - kept distinct from the visible jobTitle.
    if let Some(occ) = &site.person.occupation {
        if !occ.trim().is_empty() {
            map.insert("hasOccupation".into(), serde_json::json!({
                "@type": "Occupation",
                "name": occ,
            }));
        }
    }
    // knowsAbout: every registered entity as a stated area of expertise. The
    // sameAs link is what ties Mathew to entities Google already trusts.
    if !site.entities.is_empty() {
        let topics: Vec<serde_json::Value> = site.entities.iter().map(entity_node).collect();
        map.insert("knowsAbout".into(), topics.into());
    }
    if !site.person.same_as.is_empty() {
        map.insert("sameAs".into(), site.person.same_as.clone().into());
    }
    node
}

/// A schema.org Thing for one knowledge-graph entity, used in both the Person's
/// `knowsAbout` and a post's `about`. `sameAs` is included only when present.
fn entity_node(e: &data::Entity) -> serde_json::Value {
    let mut node = serde_json::json!({ "@type": "Thing", "name": e.name });
    if let Some(url) = &e.same_as {
        if !url.trim().is_empty() {
            node.as_object_mut().unwrap().insert("sameAs".into(), url.clone().into());
        }
    }
    node
}

/// Per-article schema.org graph: a BlogPosting (headline, dates, author,
/// publisher, image, canonical) plus a BreadcrumbList matching the on-page
/// breadcrumb. This is what powers rich article results in Google.
#[allow(clippy::too_many_arguments)]
fn article_json_ld(
    site: &data::Site,
    title: &str,
    description: &str,
    canonical_url: &str,
    published_at: &str,
    cover: Option<&str>,
    breadcrumb_href: &str,
    breadcrumb_label: &str,
    about: &[data::Entity],
) -> String {
    if site.person.name.trim().is_empty() {
        return String::new();
    }
    let person_id = format!("{}/#person", site.base_url);

    let mut posting = serde_json::json!({
        "@type": "BlogPosting",
        "headline": title,
        "description": description,
        "datePublished": published_at,
        "dateModified": published_at,
        "author": { "@id": person_id },
        "publisher": { "@id": person_id },
        "mainEntityOfPage": { "@type": "WebPage", "@id": canonical_url },
        "url": canonical_url,
        "inLanguage": "en",
    });
    if let Some(path) = cover {
        posting.as_object_mut().unwrap().insert(
            "image".into(),
            serde_json::Value::Array(vec![format!("{}{}", site.base_url, path).into()]),
        );
    }
    // `about`: the knowledge-graph entities this post covers (matched by tag).
    // Being the marked-up author of work ABOUT Weil/Camus/etc. is the durable,
    // co-citation signal Google weights above any self-applied label.
    if !about.is_empty() {
        let topics: Vec<serde_json::Value> = about.iter().map(entity_node).collect();
        posting.as_object_mut().unwrap().insert("about".into(), topics.into());
    }

    // Breadcrumb: Home -> (section, e.g. "all issues") -> this post.
    let breadcrumb = serde_json::json!({
        "@type": "BreadcrumbList",
        "itemListElement": [
            { "@type": "ListItem", "position": 1, "name": "Home",
              "item": format!("{}/", site.base_url) },
            { "@type": "ListItem", "position": 2, "name": breadcrumb_label,
              "item": format!("{}{}", site.base_url, breadcrumb_href) },
            { "@type": "ListItem", "position": 3, "name": title, "item": canonical_url },
        ],
    });

    let graph = serde_json::json!({
        "@context": "https://schema.org",
        "@graph": [ posting, person_node(site, &person_id), breadcrumb ],
    });
    to_script_json(&graph)
}

fn format_date(yyyy_mm_dd: &str) -> String {
    const MONTHS: [&str; 12] = [
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN",
        "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ];
    let year = yyyy_mm_dd.get(0..4).unwrap_or("????");
    let month_idx = yyyy_mm_dd
        .get(5..7)
        .and_then(|s| s.parse::<usize>().ok())
        .map(|m| m.saturating_sub(1).min(11))
        .unwrap_or(0);
    let day: u8 = yyyy_mm_dd
        .get(8..10)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    format!("{} {} {}", day, MONTHS[month_idx], year)
}
