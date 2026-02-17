//! HTML Sanitization for message content
//!
//! Allows safe HTML tags while preventing XSS attacks.

use ammonia::Builder;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

/// Create a configured sanitizer
fn create_sanitizer() -> Builder<'static> {
    let mut builder = Builder::default();

    // Allowed HTML tags for message formatting
    let allowed_tags: HashSet<&str> = [
        "p",
        "br", // Paragraphs and line breaks
        "strong",
        "b", // Bold
        "em",
        "i", // Italic
        "u", // Underline
        "s",
        "strike", // Strikethrough
        "a",      // Links
        "ul",
        "ol",
        "li",         // Lists
        "blockquote", // Quotes
        "code",
        "pre",  // Code
        "span", // For mentions
    ]
    .into_iter()
    .collect();

    // Set allowed tags
    builder.tags(allowed_tags);

    // Allowed attributes for specific tags
    // Note: "rel" is handled specially by ammonia via link_rel()
    let mut tag_attributes: HashMap<&str, HashSet<&str>> = HashMap::new();
    tag_attributes.insert("a", ["href", "target"].into_iter().collect());
    // Allow Tiptap mention attributes: data-type, data-id, data-label
    tag_attributes.insert(
        "span",
        [
            "data-type",
            "data-id",
            "data-label",
            "data-mention",
            "class",
        ]
        .into_iter()
        .collect(),
    );
    tag_attributes.insert("code", ["class"].into_iter().collect());
    tag_attributes.insert("pre", ["class"].into_iter().collect());

    // Set allowed attributes for each tag
    builder.tag_attributes(tag_attributes);

    // Set link rel attribute (prevents opener attacks)
    builder.link_rel(Some("noopener noreferrer"));

    // Only allow http and https URLs
    builder.url_schemes(["http", "https"].into_iter().collect());

    // Strip dangerous content
    builder.strip_comments(true);

    builder
}

static SANITIZER: Lazy<Builder<'static>> = Lazy::new(create_sanitizer);

/// Sanitize HTML content from user input
///
/// Removes dangerous elements like:
/// - script, style, iframe tags
/// - Event handlers (onclick, onerror, etc.)
/// - javascript: URLs
///
/// Preserves formatting tags:
/// - p, br, strong, em, u, s, a, ul, ol, li, blockquote, code, pre, span
pub fn sanitize_html(html: &str) -> String {
    // If content doesn't look like HTML, just escape and return
    if !html.contains('<') {
        return ammonia::clean(html);
    }

    SANITIZER.clean(html).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allows_basic_formatting() {
        let input = "<p><strong>Bold</strong> and <em>italic</em></p>";
        let output = sanitize_html(input);
        assert!(output.contains("<strong>"));
        assert!(output.contains("<em>"));
        assert!(output.contains("<p>"));
    }

    #[test]
    fn test_allows_underline_and_strike() {
        let input = "<p><u>underline</u> and <s>strike</s></p>";
        let output = sanitize_html(input);
        assert!(output.contains("<u>"));
        assert!(output.contains("<s>"));
    }

    #[test]
    fn test_allows_links() {
        let input = r#"<a href="https://example.com" target="_blank">link</a>"#;
        let output = sanitize_html(input);
        assert!(output.contains("<a"));
        assert!(output.contains("href=\"https://example.com\""));
    }

    #[test]
    fn test_removes_javascript_urls() {
        let input = r#"<a href="javascript:alert('xss')">click me</a>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("javascript:"));
    }

    #[test]
    fn test_removes_script_tags() {
        let input = "<p>Hello</p><script>alert('xss')</script>";
        let output = sanitize_html(input);
        assert!(!output.contains("<script>"));
        assert!(!output.contains("alert"));
    }

    #[test]
    fn test_removes_event_handlers() {
        let input = r#"<p onclick="alert('xss')">Click</p>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("onclick"));
        assert!(output.contains("<p>"));
    }

    #[test]
    fn test_allows_lists() {
        let input = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let output = sanitize_html(input);
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>"));
    }

    #[test]
    fn test_allows_blockquote() {
        let input = "<blockquote>Quoted text</blockquote>";
        let output = sanitize_html(input);
        assert!(output.contains("<blockquote>"));
    }

    #[test]
    fn test_allows_code() {
        let input = "<code>inline code</code><pre>code block</pre>";
        let output = sanitize_html(input);
        assert!(output.contains("<code>"));
        assert!(output.contains("<pre>"));
    }

    #[test]
    fn test_allows_mentions() {
        let input = r#"<span data-mention="uuid-123" class="mtchat-mention">@User</span>"#;
        let output = sanitize_html(input);
        assert!(output.contains("<span"));
        assert!(output.contains("data-mention"));
        assert!(output.contains("class="));
    }

    #[test]
    fn test_allows_tiptap_mentions() {
        // Tiptap Mention extension format
        let input = r#"<span data-type="mention" data-id="user-uuid-123" data-label="John Doe" class="mtchat-mention">@John Doe</span>"#;
        let output = sanitize_html(input);
        assert!(output.contains("<span"));
        assert!(output.contains("data-type=\"mention\""));
        assert!(output.contains("data-id=\"user-uuid-123\""));
        assert!(output.contains("data-label=\"John Doe\""));
        assert!(output.contains("class="));
    }

    #[test]
    fn test_plain_text_passthrough() {
        let input = "Hello, world!";
        let output = sanitize_html(input);
        assert_eq!(output, "Hello, world!");
    }

    #[test]
    fn test_escapes_html_entities() {
        let input = "<script>alert('xss')</script>";
        let output = sanitize_html(input);
        // Script tag should be removed, content preserved as text
        assert!(!output.contains("<script>"));
    }
}
