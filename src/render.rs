use askama::Template;
use serde::Serialize;
use crate::data::{Now, ConnectLink, ForgeEntry};

/// Forge entries grouped by category label, in first-seen order. The category
/// string IS the section heading, so any label renders and nothing is silently
/// dropped. (The old fixed WORK/OPEN SOURCE/CONTRIBUTING grouping hid every
/// other category - a renamed "BUSINESS" simply vanished from the page.)
pub struct ForgeGroup {
    pub label: String,
    pub entries: Vec<ForgeEntry>,
}

pub fn group_forges(entries: Vec<ForgeEntry>) -> Vec<ForgeGroup> {
    let mut groups: Vec<ForgeGroup> = Vec::new();
    for e in entries {
        match groups.iter_mut().find(|g| g.label == e.category) {
            Some(g) => g.entries.push(e),
            None => groups.push(ForgeGroup { label: e.category.clone(), entries: vec![e] }),
        }
    }
    groups
}

#[derive(Serialize)]
pub struct Quote {
    pub text: String,
    pub post_title: String,
    pub post_url: String,
    pub post_date: String,   // formatted display date of the source post
}

/// Marks an `Article` as a newsletter issue and carries its issue number for
/// the meta strip. `Some` on `Article` iff the post's kind is "newsletter".
/// (The masthead now lives in the cover image, so name/tagline aren't needed.)
pub struct NewsletterIssue {
    pub issue_display: String,  // zero-padded issue number, e.g. "001"
}

/// Everything base.html needs for a page's <head> - canonical URL, social
/// cards, and the body scope. One of these is built per rendered page.
pub struct PageMeta {
    pub canonical_url: String,   // <link rel="canonical"> and og:url
    pub body_class: String,      // "oys" on newsletter pages, else ""
    pub og_title: String,        // og:title
    pub og_description: String,  // og:description and <meta name="description">
    pub og_type: String,         // "website" or "article"
    pub og_image: String,        // absolute URL of the share image; "" omits the tag
    pub og_image_alt: String,    // og:image:alt; "" omits the tag
    pub published_time: String,  // RFC3339 article:published_time; "" omits (non-articles)
    pub json_ld: String,         // schema.org JSON-LD, already serialized; "" omits the <script>
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct Article {
    pub title: String,
    pub tags: Vec<String>,
    pub body: String,
    pub published_at_display: String,
    pub project: Option<String>,
    pub project_url: Option<String>,
    pub pr_url: Option<String>,
    pub meta: PageMeta,
    pub breadcrumb_href: String,             // where the breadcrumb points
    pub breadcrumb_label: String,            // breadcrumb text after the arrow
    pub newsletter: Option<NewsletterIssue>, // Some -> render the Own Your Stack masthead
    pub cover: Option<String>,               // site-root path; Some -> render a hero <figure>
    pub cover_alt: String,                   // alt text for the hero image
    pub related: Vec<RelatedPost>,           // sibling posts by shared tags; empty -> no block
}

/// A sibling post surfaced in the "Related" block at the foot of an article,
/// chosen by shared tags. Just enough to render a link.
pub struct RelatedPost {
    pub title: String,
    pub url: String,
    pub kind_display: String,
    pub published_at_display: String,
}

#[derive(Clone)]
pub struct Post {
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub tags: Vec<String>,
    pub published_at: String,           // raw YYYY-MM-DD for sorting (Phase 2)
    pub published_at_display: String,   // formatted for display
    pub kind: String,                   // raw slug, used as CSS modifier
    pub kind_display: String,           // uppercase, used as visible label
    pub quote: Option<String>,          // pull-quote, feeds the index rotation
    pub reading_time: u32,              // minutes, derived from body word count
    pub featured: bool,                 // sourced from frontmatter; drives FEATURED partition
    pub issue_display: Option<String>,  // zero-padded issue number; Some only for newsletters
    pub cover: Option<String>,          // site-root path to cover image; feeds the featured thumbnail
    pub body: String,                   // rendered HTML body; feeds the Atom <content>
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    pub featured_posts: Vec<Post>,   // 0-2 posts marked featured: true, most recent first
    pub posts: Vec<Post>,            // RECENT WRITING - everything else
    pub now: Now,
    pub connect: Vec<ConnectLink>,
    pub forges: Vec<ForgeGroup>,
    pub quotes_json: String,
    pub meta: PageMeta,
}

#[derive(Template)]
#[template(path = "newsletter.html")]
pub struct NewsletterIndex {
    pub meta: PageMeta,
    pub name: String,            // masthead name
    pub tagline: String,         // masthead tagline
    pub issues: Vec<Post>,       // newsletter-kind posts, newest first
}

#[derive(Template)]
#[template(path = "feed.xml", escape = "html")]
pub struct Feed {
    pub base_url: String,    // for building absolute entry URLs
    pub feed_url: String,    // absolute URL of feed.xml itself (rel="self")
    pub home_url: String,    // absolute URL of /newsletter/ (the feed's id + alternate)
    pub title: String,       // newsletter name
    pub subtitle: String,    // newsletter tagline
    pub updated: String,     // RFC3339 timestamp of the most recent issue
    pub entries: Vec<Post>,  // newsletter issues, newest first
}

pub struct SitemapUrl {
    pub loc: String,             // absolute URL of the page
    pub lastmod: Option<String>, // YYYY-MM-DD; omitted from the XML when None
}

#[derive(Template)]
#[template(path = "sitemap.xml", escape = "html")]
pub struct Sitemap {
    pub urls: Vec<SitemapUrl>,
}
