//! Serves the built front end (`WEB_DIR`) and builds the Content-Security-Policy for it.
//!
//! SvelteKit starts the app with one inline script in `index.html`. The policy allows exactly that
//! script by its SHA-256 hash; every other inline script stays blocked. Inline styles are allowed
//! (SvelteKit sets a `style` attribute on its page-change announcer for screen readers); styles
//! cannot run code.

use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Web {
    pub dir: PathBuf,
    pub content_security_policy: String,
}

impl Web {
    /// Reads `index.html` once at start. Without it (for example in tests of the API only) the
    /// policy allows no inline scripts, and pages are not found.
    pub fn load(dir: &Path) -> Self {
        let hashes = match std::fs::read_to_string(dir.join("index.html")) {
            Ok(index) => inline_script_hashes(&index),
            Err(error) => {
                tracing::warn!(dir = %dir.display(), %error, "no front end found");
                Vec::new()
            }
        };
        Self {
            dir: dir.to_owned(),
            content_security_policy: content_security_policy(&hashes),
        }
    }

    pub fn index(&self) -> PathBuf {
        self.dir.join("index.html")
    }
}

pub fn content_security_policy(script_hashes: &[String]) -> String {
    let mut script_src = String::from("'self'");
    for hash in script_hashes {
        script_src.push_str(&format!(" 'sha256-{hash}'"));
    }
    format!(
        "default-src 'self'; script-src {script_src}; style-src 'self' 'unsafe-inline'; \
         frame-ancestors 'none'"
    )
}

/// Base64 SHA-256 hashes of the content of every `<script>` without a `src` attribute.
pub fn inline_script_hashes(html: &str) -> Vec<String> {
    let mut hashes = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<script") {
        let after_start = &rest[start..];
        let Some(tag_end) = after_start.find('>') else {
            break;
        };
        let tag = &after_start[..tag_end];
        let body_and_rest = &after_start[tag_end + 1..];
        let Some(close) = body_and_rest.find("</script>") else {
            break;
        };
        if !tag.contains("src=") {
            let body = &body_and_rest[..close];
            hashes.push(STANDARD.encode(Sha256::digest(body.as_bytes())));
        }
        rest = &body_and_rest[close + "</script>".len()..];
    }
    hashes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_only_inline_scripts() {
        let html =
            r#"<head><script src="/a.js"></script></head><body><script>start()</script></body>"#;
        let expected = STANDARD.encode(Sha256::digest(b"start()"));
        assert_eq!(inline_script_hashes(html), vec![expected]);
    }

    #[test]
    fn policy_lists_the_hashes() {
        assert_eq!(
            content_security_policy(&["abc=".to_owned()]),
            "default-src 'self'; script-src 'self' 'sha256-abc='; \
             style-src 'self' 'unsafe-inline'; frame-ancestors 'none'"
        );
        assert_eq!(
            content_security_policy(&[]),
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
             frame-ancestors 'none'"
        );
    }
}
