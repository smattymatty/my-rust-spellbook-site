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
    let forges = render::ForgesGrouped::from_flat(forges_flat);

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

    // Phase 3: render every non-draft file
    let mut posts = Vec::new();
    for (path, doc) in documents {
        let post = render_markdown_to_html(
            &path, doc, content_root, output_root, &site, &newsletter,
        )
        .with_context(|| format!("rendering {}", path.display()))?;
        posts.push(post);
    }

    let post_count = posts.len();

    // Collect pull-quotes from every post that has one, for the index rotation panel.
    let quotes: Vec<render::Quote> = posts.iter()
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

    // Sitemap entries - every page. Built before the partition consumes `posts`.
    let mut sitemap_urls = vec![
        render::SitemapUrl { loc: format!("{}/", site.base_url), lastmod: None },
        render::SitemapUrl { loc: format!("{}/newsletter/", site.base_url), lastmod: None },
    ];
    for p in &posts {
        sitemap_urls.push(render::SitemapUrl {
            loc: format!("{}{}", site.base_url, p.url),
            lastmod: Some(p.published_at.clone()),
        });
    }

    // Partition into featured + recent. Both sorted most-recent first; featured capped at 2.
    let (mut featured_all, mut recent_posts): (Vec<_>, Vec<_>) = posts
        .into_iter()
        .partition(|p| p.featured);
    featured_all.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    recent_posts.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    let featured_posts: Vec<_> = featured_all.into_iter().take(2).collect();

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
        },
        name: newsletter.name.clone(),
        tagline: newsletter.tagline.clone(),
        issues: newsletter_issues.clone(),
    };
    let newsletter_html = newsletter_page.render().context("rendering newsletter index")?;
    let written = output::write_at(output_root, "newsletter/index.html", &newsletter_html)?;
    println!("wrote {} ({} issue(s))", written.display(), newsletter_page.issues.len());

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

fn render_markdown_to_html(
    content_path: &Path,
    doc: content::Document,
    content_root: &Path,
    output_root: &Path,
    site: &data::Site,
    newsletter_meta: &data::Newsletter,
) -> Result<render::Post> {
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
    let (breadcrumb_href, breadcrumb_label) = if is_newsletter {
        ("/newsletter/".to_string(), "all issues".to_string())
    } else {
        ("/".to_string(), "all writing".to_string())
    };
    let newsletter = is_newsletter.then(|| render::NewsletterIssue {
        name: newsletter_meta.name.clone(),
        tagline: newsletter_meta.tagline.clone(),
        // The validator guarantees a newsletter issue carries `issue`.
        issue_display: issue_display.clone().unwrap_or_default(),
    });

    let meta = render::PageMeta {
        canonical_url: format!("{}{}", site.base_url, url),
        body_class,
        og_title: title.clone(),
        og_description: description.clone().unwrap_or_else(|| site.description.clone()),
        og_type: "article".to_string(),
        og_image: og_image_url(site),
    };

    let article = render::Article {
        title: doc.frontmatter.title,
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
    };
    let page_html = article.render().context("rendering article template")?;

    let written = output::write_html(content_root, output_root, content_path, &page_html)?;
    println!("wrote {} ({})", written.display(), article.title);

    Ok(render::Post {
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
        body: body_html,
    })
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
