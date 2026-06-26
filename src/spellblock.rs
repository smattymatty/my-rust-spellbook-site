//! SpellBlock rendering - Django Spellbook's `{~ name attr="v" ~}…{~~}` block
//! syntax inside markdown.
//!
//! The markdown pipeline calls [`render`] instead of pulldown-cmark directly:
//! the document is split into plain-markdown segments and block segments, each
//! is rendered, and the results are concatenated. A block's body is itself
//! markdown. v1 blocks: alert, card, label_seperator, accordion - adding one is
//! a single arm of [`render_block`]. Unknown or nested blocks fail the build.

use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use pulldown_cmark::{html, CowStr, Event, Parser, Tag, TagEnd};

/// Render a markdown document - expanding SpellBlocks - into an HTML string.
pub fn render(markdown: &str) -> Result<String> {
    let mut out = String::new();
    // Tracks heading slugs across the whole document so duplicate titles get
    // suffixed ids (-2, -3) rather than colliding. Only top-level body headings
    // are anchored; headings inside SpellBlocks render plain.
    let mut seen = HashMap::new();
    for segment in split(markdown)? {
        match segment {
            Segment::Markdown(md) => out.push_str(&markdown_to_html_anchored(&md, &mut seen)),
            Segment::Block { name, attrs, inner } => {
                out.push_str(&render_block(&name, &attrs, &inner)?);
            }
        }
    }
    Ok(out)
}

enum Segment {
    Markdown(String),
    Block {
        name: String,
        attrs: HashMap<String, String>,
        inner: String,
    },
}

/// Split raw markdown into an ordered run of plain-markdown and block segments.
/// Fails loud on an unclosed tag, a stray `{~~}`, or a nested block.
fn split(src: &str) -> Result<Vec<Segment>> {
    let mut segments = Vec::new();
    let mut markdown = String::new();
    let mut rest = src;

    while let Some(at) = find_tag_outside_fence(rest) {
        markdown.push_str(&rest[..at]);
        let (body, after) = tag_at(&rest[at..])?;
        if body.is_empty() {
            bail!("SpellBlock: a `{{~~}}` close with no matching block open");
        }

        // An opening tag - flush the markdown collected so far.
        if !markdown.is_empty() {
            segments.push(Segment::Markdown(std::mem::take(&mut markdown)));
        }
        let (name, attrs) = parse_tag(body)?;

        // Collect inner content up to the matching `{~~}` close.
        let mut inner = String::new();
        let mut cursor = after;
        loop {
            let next = cursor.find("{~").ok_or_else(|| {
                anyhow!("SpellBlock: block `{name}` is opened but never closed with `{{~~}}`")
            })?;
            inner.push_str(&cursor[..next]);
            let (tag_body, tail) = tag_at(&cursor[next..])?;
            if tag_body.is_empty() {
                cursor = tail;
                break;
            }
            bail!("SpellBlock: block `{tag_body}` nested inside `{name}` - nesting is not supported");
        }
        segments.push(Segment::Block { name, attrs, inner });
        rest = cursor;
    }

    markdown.push_str(rest);
    if !markdown.is_empty() {
        segments.push(Segment::Markdown(markdown));
    }
    Ok(segments)
}

/// Byte offset of the next `{~` that is NOT inside a fenced code block
/// (``` or ~~~), or None. This lets a post show literal SpellBlock syntax in a
/// code fence - e.g. a guide documenting the syntax - without it being expanded.
fn find_tag_outside_fence(s: &str) -> Option<usize> {
    let mut offset = 0usize;
    let mut fence: Option<&str> = None; // the marker (``` or ~~~) that opened the open fence
    for line in s.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let marker = if trimmed.starts_with("```") {
            Some("```")
        } else if trimmed.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };
        match fence {
            // Inside a fence: ignore `{~`; only a matching marker closes it.
            Some(open) => {
                if marker == Some(open) {
                    fence = None;
                }
            }
            // Outside a fence: a marker opens one; otherwise scan for `{~`.
            None => match marker {
                Some(m) => fence = Some(m),
                None => {
                    if let Some(pos) = line.find("{~") {
                        return Some(offset + pos);
                    }
                }
            },
        }
        offset += line.len();
    }
    None
}

/// Given a slice that starts with `{~`, return the trimmed tag body and the
/// remainder after the closing `~}`. An empty body is a `{~~}` close marker.
fn tag_at(s: &str) -> Result<(&str, &str)> {
    let inside = &s[2..];
    let end = inside
        .find("~}")
        .ok_or_else(|| anyhow!("SpellBlock: `{{~` opened with no closing `~}}`"))?;
    Ok((inside[..end].trim(), &inside[end + 2..]))
}

/// Parse a tag body (`name attr="v" attr2='v2'`) into a name and attributes.
/// Quoted values may contain the other quote character.
fn parse_tag(body: &str) -> Result<(String, HashMap<String, String>)> {
    let (name, mut rest) = match body.find(char::is_whitespace) {
        Some(i) => (&body[..i], body[i..].trim_start()),
        None => (body, ""),
    };
    if name.is_empty() {
        bail!("SpellBlock: a tag with no block name");
    }

    let mut attrs = HashMap::new();
    while !rest.is_empty() {
        let eq = rest
            .find('=')
            .ok_or_else(|| anyhow!("SpellBlock `{name}`: malformed attribute near `{rest}`"))?;
        let key = rest[..eq].trim().to_string();
        let after_eq = rest[eq + 1..].trim_start();
        let quote = after_eq
            .chars()
            .next()
            .filter(|c| *c == '"' || *c == '\'')
            .ok_or_else(|| anyhow!("SpellBlock `{name}`: value of `{key}` must be quoted"))?;
        let value = &after_eq[1..];
        let end = value
            .find(quote)
            .ok_or_else(|| anyhow!("SpellBlock `{name}`: unterminated value for `{key}`"))?;
        attrs.insert(key, value[..end].to_string());
        rest = value[end + 1..].trim_start();
    }
    Ok((name.to_string(), attrs))
}

fn markdown_to_html(md: &str) -> String {
    let mut out = String::new();
    html::push_html(&mut out, Parser::new(md));
    out
}

/// Like [`markdown_to_html`] but stamps every heading with a slug `id` and
/// splices a permalink anchor (`<a class="heading-anchor" href="#slug">#</a>`)
/// in just before the closing tag, so sections are linkable. Used for the
/// article body only; `seen` de-duplicates slugs across the document.
fn markdown_to_html_anchored(md: &str, seen: &mut HashMap<String, usize>) -> String {
    let mut events: Vec<Event> = Parser::new(md).collect();

    let mut i = 0;
    while i < events.len() {
        // Pull the heading's parts out by value so the borrow ends before we
        // mutate the vector below.
        let head = match &events[i] {
            Event::Start(Tag::Heading { level, classes, attrs, .. }) => {
                Some((*level, classes.clone(), attrs.clone()))
            }
            _ => None,
        };
        let Some((level, classes, attrs)) = head else {
            i += 1;
            continue;
        };

        // Walk to the matching close, collecting visible text for the slug.
        let mut text = String::new();
        let mut end = i + 1;
        while end < events.len() {
            match &events[end] {
                Event::Text(t) | Event::Code(t) => text.push_str(t),
                Event::End(TagEnd::Heading(_)) => break,
                _ => {}
            }
            end += 1;
        }

        let slug = unique_slug(&slugify(&text), seen);

        // Mark the heading so CSS targets only article-body headings, never the
        // template's own (e.g. the "Related" title).
        let mut classes = classes;
        classes.push(CowStr::from("anchored"));
        events[i] = Event::Start(Tag::Heading {
            level,
            id: Some(CowStr::from(slug.clone())),
            classes,
            attrs,
        });

        // Wrap the heading's whole text in a permalink link, so clicking
        // anywhere on the heading jumps to it. The "#" marker is drawn by CSS.
        events.insert(
            i + 1,
            Event::Html(CowStr::from(format!("<a class=\"heading-link\" href=\"#{slug}\">"))),
        );
        // The opening insert shifted the closing heading tag one slot right.
        events.insert(end + 1, Event::Html(CowStr::from("</a>")));

        // Resume past: start, open-link, …text…, close-link, end.
        i = end + 3;
    }

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    out
}

/// Turn heading text into a URL-safe slug: lowercase, alphanumerics kept,
/// runs of everything else collapsed to single dashes, no leading/trailing
/// dash. Empty results fall back to "section".
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(c.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    if slug.is_empty() {
        slug.push_str("section");
    }
    slug
}

/// Disambiguate a slug against ones already used in the document, appending
/// `-2`, `-3`, … on collision.
fn unique_slug(base: &str, seen: &mut HashMap<String, usize>) -> String {
    let count = seen.entry(base.to_string()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base.to_string()
    } else {
        format!("{base}-{count}")
    }
}

/// Dispatch a parsed block to its renderer. An unknown name fails the build.
fn render_block(name: &str, attrs: &HashMap<String, String>, inner: &str) -> Result<String> {
    let inner_html = markdown_to_html(inner);
    match name {
        "alert" => alert(attrs, &inner_html),
        "card" => Ok(card(attrs, &inner_html)),
        "label_seperator" => Ok(label_seperator(attrs, &inner_html)),
        "accordion" => Ok(accordion(attrs, &inner_html)),
        other => bail!(
            "SpellBlock: unknown block `{other}` - v1 supports alert, card, label_seperator, accordion"
        ),
    }
}

// ─── Block renderers - native to mathewstorm.ca; styles live in style.css ───

const ALERT_TYPES: [(&str, &str); 4] = [
    ("info", "ℹ️"),
    ("warning", "⚠️"),
    ("success", "✅"),
    ("danger", "🚫"),
];

fn alert(attrs: &HashMap<String, String>, inner_html: &str) -> Result<String> {
    let kind = attrs.get("type").map(String::as_str).unwrap_or("info");
    let icon = ALERT_TYPES
        .iter()
        .find(|(n, _)| *n == kind)
        .map(|(_, icon)| *icon)
        .ok_or_else(|| {
            anyhow!("SpellBlock alert: unknown type `{kind}` - use info, warning, success, or danger")
        })?;
    Ok(format!(
        "<aside class=\"spell-callout spell-callout--{kind}\">\n\
         <span class=\"spell-callout-icon\" aria-hidden=\"true\">{icon}</span>\n\
         <div class=\"spell-callout-body\">\n{inner_html}</div>\n\
         </aside>\n"
    ))
}

fn card(attrs: &HashMap<String, String>, inner_html: &str) -> String {
    let head = attrs
        .get("title")
        .map(|t| format!("<header class=\"spell-card-head\">{}</header>\n", escape(t)))
        .unwrap_or_default();
    let foot = attrs
        .get("footer")
        .map(|f| format!("<footer class=\"spell-card-foot\">{}</footer>\n", escape(f)))
        .unwrap_or_default();
    format!(
        "<article class=\"spell-card\">\n{head}\
         <div class=\"spell-card-body\">\n{inner_html}</div>\n{foot}\
         </article>\n"
    )
}

fn label_seperator(attrs: &HashMap<String, String>, inner_html: &str) -> String {
    // `color` becomes a CSS modifier class; keep it to a safe token.
    let color: String = attrs
        .get("color")
        .map(String::as_str)
        .unwrap_or("default")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    format!(
        "<div class=\"spell-step spell-step--{color}\">\n\
         <span class=\"spell-step-label\">{inner_html}</span>\n\
         </div>\n"
    )
}

fn accordion(attrs: &HashMap<String, String>, inner_html: &str) -> String {
    let title = escape(attrs.get("title").map(String::as_str).unwrap_or(""));
    format!(
        "<details class=\"spell-accordion\">\n\
         <summary class=\"spell-accordion-summary\">{title}</summary>\n\
         <div class=\"spell-accordion-body\">\n{inner_html}</div>\n\
         </details>\n"
    )
}

/// Escape text bound for HTML text or attribute content.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_markdown_passes_through() {
        let out = render("# Title\n\nbody text").unwrap();
        // Headings are now slugged and their text wrapped in a permalink link.
        assert!(out.contains("<h1 id=\"title\" class=\"anchored\">"));
        assert!(out.contains("<a class=\"heading-link\" href=\"#title\">Title</a>"));
        assert!(out.contains("<p>body text</p>"));
    }

    #[test]
    fn duplicate_heading_titles_get_unique_slugs() {
        let out = render("## Setup\n\ntext\n\n## Setup\n\nmore").unwrap();
        assert!(out.contains("id=\"setup\""));
        assert!(out.contains("id=\"setup-2\""));
    }

    #[test]
    fn slugify_collapses_punctuation_and_spaces() {
        assert_eq!(slugify("Where the Workflows Go"), "where-the-workflows-go");
        assert_eq!(slugify("What a Tag *Actually* Promises!"), "what-a-tag-actually-promises");
        assert_eq!(slugify("  ---  "), "section");
    }

    #[test]
    fn alert_renders_type_icon_and_inner_markdown() {
        let out = render("{~ alert type=\"warning\" ~}\n**heed**\n{~~}").unwrap();
        assert!(out.contains("spell-callout--warning"));
        assert!(out.contains("⚠️"));
        assert!(out.contains("<strong>heed</strong>"));
    }

    #[test]
    fn card_carries_title_and_footer() {
        let out = render("{~ card title=\"T\" footer=\"F\" ~}\nbody\n{~~}").unwrap();
        assert!(out.contains("spell-card-head\">T</header>"));
        assert!(out.contains("spell-card-foot\">F</footer>"));
        assert!(out.contains("<p>body</p>"));
    }

    #[test]
    fn accordion_is_a_native_details_element() {
        let out = render("{~ accordion title=\"More\" ~}\nhidden\n{~~}").unwrap();
        assert!(out.contains("<details class=\"spell-accordion\">"));
        assert!(out.contains("spell-accordion-summary\">More</summary>"));
    }

    #[test]
    fn label_seperator_renders_its_label() {
        let out = render("{~ label_seperator color=\"primary\" ~}\n**Step 1**\n{~~}").unwrap();
        assert!(out.contains("spell-step--primary"));
        assert!(out.contains("<strong>Step 1</strong>"));
    }

    #[test]
    fn attribute_value_may_contain_the_other_quote() {
        let (name, attrs) = parse_tag("card title='a \"quoted\" bit'").unwrap();
        assert_eq!(name, "card");
        assert_eq!(attrs["title"], "a \"quoted\" bit");
    }

    #[test]
    fn surrounding_markdown_and_block_both_render() {
        let out = render("before\n\n{~ alert ~}\nin\n{~~}\n\nafter").unwrap();
        assert!(out.contains("<p>before</p>"));
        assert!(out.contains("spell-callout"));
        assert!(out.contains("<p>after</p>"));
    }

    #[test]
    fn unknown_block_fails_loud() {
        assert!(render("{~ hero ~}\nx\n{~~}").is_err());
    }

    #[test]
    fn nested_block_fails_loud() {
        assert!(render("{~ card ~}\n{~ alert ~}\ny\n{~~}\n{~~}").is_err());
    }

    #[test]
    fn unclosed_block_fails_loud() {
        assert!(render("{~ card ~}\nno close").is_err());
    }

    #[test]
    fn bad_alert_type_fails_loud() {
        assert!(render("{~ alert type=\"nope\" ~}\nx\n{~~}").is_err());
    }
}
