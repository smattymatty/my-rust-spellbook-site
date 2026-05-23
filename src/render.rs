use askama::Template;
use serde::Serialize;
use crate::data::{Now, ConnectLink, ForgeEntry};

pub struct ForgesGrouped {
    pub work: Vec<ForgeEntry>,
    pub open_source: Vec<ForgeEntry>,
    pub contributing: Vec<ForgeEntry>,
}

impl ForgesGrouped {
    pub fn from_flat(entries: Vec<ForgeEntry>) -> Self {
        let mut work = Vec::new();
        let mut open_source = Vec::new();
        let mut contributing = Vec::new();
        for e in entries {
            match e.category.as_str() {
                "WORK" => work.push(e),
                "OPEN SOURCE" => open_source.push(e),
                "CONTRIBUTING" => contributing.push(e),
                _ => {} // unknown category; silently dropped (could become validator concern later)
            }
        }
        Self { work, open_source, contributing }
    }
}

#[derive(Serialize)]
pub struct Quote {
    pub text: String,
    pub post_title: String,
    pub post_url: String,
    pub post_date: String,   // formatted display date of the source post
}

/// The "Own Your Stack" identity carried by a newsletter issue page.
/// `Some` on `Article` iff the post's kind is "newsletter".
pub struct NewsletterIssue {
    pub name: String,           // "Own Your Stack" — masthead
    pub tagline: String,        // masthead tagline
    pub issue_display: String,  // zero-padded issue number, e.g. "001"
}

/// Everything base.html needs for a page's <head> — canonical URL, social
/// cards, and the body scope. One of these is built per rendered page.
pub struct PageMeta {
    pub canonical_url: String,   // <link rel="canonical"> and og:url
    pub body_class: String,      // "oys" on newsletter pages, else ""
    pub og_title: String,        // og:title
    pub og_description: String,  // og:description and <meta name="description">
    pub og_type: String,         // "website" or "article"
    pub og_image: String,        // absolute URL of the share image; "" omits the tag
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
    pub body: String,                   // rendered HTML body; feeds the Atom <content>
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    pub featured_posts: Vec<Post>,   // 0-2 posts marked featured: true, most recent first
    pub posts: Vec<Post>,            // RECENT WRITING — everything else
    pub now: Now,
    pub connect: Vec<ConnectLink>,
    pub forges: ForgesGrouped,
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
