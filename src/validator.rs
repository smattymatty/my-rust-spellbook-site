use std::path::Path;
use crate::content::Frontmatter;

const ALLOWED_KINDS: &[&str] = &["post", "newsletter", "contribution", "dispatch", "tutorial"];

pub struct ValidationError {
    pub file: String,
    pub field: &'static str,
    pub problem: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: `{}` {}", self.file, self.field, self.problem)
    }
}

pub fn validate(frontmatter: &Frontmatter, path: &Path) -> Vec<ValidationError> {
    let file = path.display().to_string();
    let mut errors = Vec::new();

    if frontmatter.title.trim().is_empty() {
        errors.push(ValidationError {
            file: file.clone(),
            field: "title",
            problem: "is missing or empty".to_string(),
        });
    }

    let date = frontmatter.published_at.trim();
    if date.is_empty() {
        errors.push(ValidationError {
            file: file.clone(),
            field: "published_at",
            problem: "is missing or empty".to_string(),
        });
    } else if !is_yyyy_mm_dd(date) {
        errors.push(ValidationError {
            file: file.clone(),
            field: "published_at",
            problem: format!("'{}' is not a valid YYYY-MM-DD date", date),
        });
    }

    if frontmatter.tags.is_empty() {
        errors.push(ValidationError {
            file: file.clone(),
            field: "tags",
            problem: "must have at least one tag".to_string(),
        });
    }

    if !ALLOWED_KINDS.contains(&frontmatter.kind.as_str()) {
        errors.push(ValidationError {
            file: file.clone(),
            field: "kind",
            problem: format!(
                "'{}' is not one of: {}",
                frontmatter.kind,
                ALLOWED_KINDS.join(", "),
            ),
        });
    }

    if frontmatter.kind == "newsletter" && frontmatter.issue.is_none() {
        errors.push(ValidationError {
            file: file.clone(),
            field: "issue",
            problem: "is required when kind is newsletter".to_string(),
        });
    }

    if let Some(q) = &frontmatter.quote {
        if q.trim().is_empty() {
            errors.push(ValidationError {
                file: file.clone(),
                field: "quote",
                problem: "is present but empty".to_string(),
            });
        }
    }

    if frontmatter.kind == "contribution" {
        if frontmatter.project.is_none() {
            errors.push(ValidationError {
                file: file.clone(),
                field: "project",
                problem: "is required when kind is contribution".to_string(),
            });
        }
        if frontmatter.pr_url.is_none() {
            errors.push(ValidationError {
                file,
                field: "pr_url",
                problem: "is required when kind is contribution".to_string(),
            });
        }
    }

    errors
}

fn is_yyyy_mm_dd(s: &str) -> bool {
    if s.len() != 10 { return false; }
    let bytes = s.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' { return false; }
    let year  = &s[0..4];
    let month = &s[5..7];
    let day   = &s[8..10];

    let all_digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    if !all_digits(year) || !all_digits(month) || !all_digits(day) {
        return false;
    }
    let m: u8 = month.parse().unwrap_or(0);
    let d: u8 = day.parse().unwrap_or(0);
    (1..=12).contains(&m) && (1..=31).contains(&d)
}
