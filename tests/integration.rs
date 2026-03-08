use std::process::Command;

fn forge(url: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .arg(url)
        .output()
        .expect("Failed to run forge");
    assert!(output.status.success(), "forge exited with error for {url}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).expect("Invalid UTF-8 output")
}

// =========================================================================
// Binary basics
// =========================================================================

#[test]
fn no_args_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .output()
        .expect("Failed to run forge");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage"));
}

#[test]
fn invalid_url_exits_with_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .arg("not-a-url")
        .output()
        .expect("Failed to run forge");
    assert!(!output.status.success());
}

// =========================================================================
// Wikipedia: Stigmergy
// =========================================================================

#[test]
fn wikipedia_stigmergy_has_title() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    assert!(md.contains("# Stigmergy"), "Missing h1 title");
}

#[test]
fn wikipedia_stigmergy_has_sections() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    assert!(md.contains("## History"), "Missing History section");
}

#[test]
fn wikipedia_stigmergy_has_links() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    assert!(md.contains("](/wiki/"), "Missing internal wiki links");
}

#[test]
fn wikipedia_stigmergy_no_scripts() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    assert!(!md.contains("<script"), "Script tags leaked through");
    assert!(!md.contains("function("), "JS code leaked through");
}

#[test]
fn wikipedia_stigmergy_no_html_tags() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    assert!(!md.contains("<div"), "div tags leaked");
    assert!(!md.contains("<span"), "span tags leaked");
    assert!(!md.contains("<p>"), "p tags leaked");
    assert!(!md.contains("</p>"), "closing p tags leaked");
}

#[test]
fn wikipedia_stigmergy_has_bold() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    assert!(md.contains("**Stigmergy**"), "Missing bold term definition");
}

#[test]
fn wikipedia_stigmergy_has_references() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    // Wikipedia references show up as superscript links
    assert!(md.contains("^("), "Missing superscript references");
}

#[test]
fn wikipedia_stigmergy_no_nav() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    // The sidebar navigation should be stripped
    assert!(!md.contains("Main page"), "Nav sidebar content leaked");
}

// =========================================================================
// Hacker News
// =========================================================================

#[test]
fn hn_has_content() {
    let md = forge("https://news.ycombinator.com");
    assert!(!md.is_empty(), "HN returned empty");
    assert!(md.len() > 100, "HN output suspiciously short");
}

#[test]
fn hn_has_links() {
    let md = forge("https://news.ycombinator.com");
    assert!(md.contains("]("), "HN should contain markdown links");
}

#[test]
fn hn_no_scripts() {
    let md = forge("https://news.ycombinator.com");
    assert!(!md.contains("<script"), "Script tags in HN output");
    assert!(!md.contains("document."), "JS leaked in HN output");
}

// =========================================================================
// Rust Blog
// =========================================================================

#[test]
fn rust_blog_has_content() {
    let md = forge("https://blog.rust-lang.org");
    assert!(!md.is_empty(), "Rust blog returned empty");
}

#[test]
fn rust_blog_mentions_rust() {
    let md = forge("https://blog.rust-lang.org");
    assert!(md.to_lowercase().contains("rust"), "Rust blog should mention Rust");
}

#[test]
fn rust_blog_has_links() {
    let md = forge("https://blog.rust-lang.org");
    assert!(md.contains("]("), "Rust blog should have markdown links");
}

#[test]
fn rust_blog_no_html() {
    let md = forge("https://blog.rust-lang.org");
    assert!(!md.contains("<div"), "HTML tags leaked in Rust blog output");
    assert!(!md.contains("<script"), "Script tags in Rust blog output");
}

// =========================================================================
// Output quality
// =========================================================================

#[test]
fn output_ends_with_newline() {
    let md = forge("https://blog.rust-lang.org");
    assert!(md.ends_with('\n'), "Output should end with newline");
}

#[test]
fn no_excessive_blank_lines() {
    let md = forge("https://en.wikipedia.org/wiki/Stigmergy");
    assert!(!md.contains("\n\n\n\n"), "More than 3 consecutive newlines found");
}

#[test]
fn no_trailing_whitespace_on_lines() {
    let md = forge("https://blog.rust-lang.org");
    for (i, line) in md.lines().enumerate() {
        assert!(
            line == line.trim_end(),
            "Line {i} has trailing whitespace: {:?}",
            line
        );
    }
}
