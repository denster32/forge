use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::env;
use std::io::Read;
use std::process;

fn main() {
    let url = match env::args().nth(1) {
        Some(u) => u,
        None => {
            eprintln!("Usage: forge <url>");
            process::exit(1);
        }
    };

    let html = fetch(&url);
    let dom = parse_html(&html);
    let root = find_content(&dom.document);
    let md = to_markdown(&root);
    let cleaned = clean_output(&md);
    print!("{}", cleaned);
}

fn fetch(url: &str) -> String {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Forge/0.1)")
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to build HTTP client: {e}");
            process::exit(1);
        });

    let mut resp = client.get(url).send().unwrap_or_else(|e| {
        eprintln!("Failed to fetch URL: {e}");
        process::exit(1);
    });

    let mut body = String::new();
    resp.read_to_string(&mut body).unwrap_or_else(|e| {
        eprintln!("Failed to read response: {e}");
        process::exit(1);
    });
    body
}

fn parse_html(html: &str) -> RcDom {
    let opts = ParseOpts::default();
    parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .expect("Failed to parse HTML")
}

const STRIP_TAGS: &[&str] = &[
    "script", "style", "nav", "footer", "aside", "iframe", "svg", "canvas",
    "form", "noscript", "template",
];

fn should_strip(tag: &str) -> bool {
    STRIP_TAGS.contains(&tag)
}

fn is_hidden(node: &Handle) -> bool {
    if let NodeData::Element { ref attrs, .. } = node.data {
        let attrs = attrs.borrow();
        for attr in attrs.iter() {
            let name = &*attr.name.local;
            let val = attr.value.to_lowercase();
            if name == "aria-hidden" && val == "true" {
                return true;
            }
            if name == "hidden" {
                return true;
            }
            if name == "style" {
                if val.contains("display:none") || val.contains("display: none")
                    || val.contains("visibility:hidden") || val.contains("visibility: hidden")
                {
                    return true;
                }
            }
            if name == "class" || name == "id" {
                if val.contains("cookie") || val.contains("consent") || val.contains("banner")
                    || val.contains("modal") || val.contains("popup") || val.contains("overlay")
                    || val.contains("ad-") || val.contains("ads-") || val.contains("advert")
                    || val.contains("tracking") || val.contains("newsletter-popup")
                {
                    return true;
                }
            }
            if name == "role" {
                if val == "banner" || val == "navigation" || val == "dialog" || val == "alertdialog" {
                    return true;
                }
            }
        }
    }
    false
}

fn find_content(doc: &Handle) -> Handle {
    if let Some(article) = find_tag(doc, "article") {
        return article;
    }
    if let Some(main) = find_tag(doc, "main") {
        return main;
    }
    if let Some(body) = find_tag(doc, "body") {
        return body;
    }
    doc.clone()
}

fn find_tag(node: &Handle, target: &str) -> Option<Handle> {
    if let NodeData::Element { ref name, .. } = node.data {
        if &*name.local == target {
            return Some(node.clone());
        }
    }
    for child in node.children.borrow().iter() {
        if let Some(found) = find_tag(child, target) {
            return Some(found);
        }
    }
    None
}

fn get_attr(node: &Handle, attr_name: &str) -> Option<String> {
    if let NodeData::Element { ref attrs, .. } = node.data {
        for attr in attrs.borrow().iter() {
            if &*attr.name.local == attr_name {
                return Some(attr.value.to_string());
            }
        }
    }
    None
}

fn tag_name(node: &Handle) -> Option<String> {
    if let NodeData::Element { ref name, .. } = node.data {
        Some(name.local.to_string())
    } else {
        None
    }
}

fn to_markdown(node: &Handle) -> String {
    let mut out = String::new();
    convert_node(node, &mut out, false, 0);
    out
}

fn convert_node(node: &Handle, out: &mut String, in_pre: bool, list_depth: usize) {
    if is_hidden(node) {
        return;
    }

    match node.data {
        NodeData::Text { ref contents } => {
            let text = contents.borrow().to_string();
            if in_pre {
                out.push_str(&text);
            } else {
                let collapsed = collapse_whitespace(&text);
                if !collapsed.is_empty() {
                    out.push_str(&collapsed);
                }
            }
        }
        NodeData::Element { ref name, .. } => {
            let tag = &*name.local;

            if should_strip(tag) {
                return;
            }

            match tag {
                "h1" => {
                    out.push_str("\n\n# ");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "h2" => {
                    out.push_str("\n\n## ");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "h3" => {
                    out.push_str("\n\n### ");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "h4" => {
                    out.push_str("\n\n#### ");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "h5" => {
                    out.push_str("\n\n##### ");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "h6" => {
                    out.push_str("\n\n###### ");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "p" => {
                    out.push_str("\n\n");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "br" => {
                    out.push('\n');
                }
                "hr" => {
                    out.push_str("\n\n---\n\n");
                }
                "a" => {
                    let href = get_attr(node, "href").unwrap_or_default();
                    out.push('[');
                    convert_children(node, out, false, list_depth);
                    out.push_str("](");
                    out.push_str(&href);
                    out.push(')');
                }
                "img" => {
                    let alt = get_attr(node, "alt").unwrap_or_else(|| "Image".to_string());
                    let src = get_attr(node, "src").unwrap_or_default();
                    out.push_str(&format!("![{alt}]({src})"));
                }
                "strong" | "b" => {
                    out.push_str("**");
                    convert_children(node, out, false, list_depth);
                    out.push_str("**");
                }
                "em" | "i" => {
                    out.push('*');
                    convert_children(node, out, false, list_depth);
                    out.push('*');
                }
                "code" => {
                    if in_pre {
                        convert_children(node, out, true, list_depth);
                    } else {
                        out.push('`');
                        convert_children(node, out, false, list_depth);
                        out.push('`');
                    }
                }
                "pre" => {
                    out.push_str("\n\n```\n");
                    convert_children(node, out, true, list_depth);
                    out.push_str("\n```\n\n");
                }
                "blockquote" => {
                    out.push_str("\n\n");
                    let mut inner = String::new();
                    convert_children(node, &mut inner, false, list_depth);
                    for line in inner.trim().lines() {
                        out.push_str("> ");
                        out.push_str(line);
                        out.push('\n');
                    }
                    out.push('\n');
                }
                "ul" | "ol" => {
                    out.push('\n');
                    convert_children(node, out, false, list_depth + 1);
                    out.push('\n');
                }
                "li" => {
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    out.push_str(&indent);
                    out.push_str("- ");
                    convert_children(node, out, false, list_depth);
                    out.push('\n');
                }
                "table" => {
                    out.push_str("\n\n");
                    convert_table(node, out);
                    out.push_str("\n\n");
                }
                "figure" => {
                    out.push_str("\n\n");
                    convert_children(node, out, false, list_depth);
                    out.push_str("\n\n");
                }
                "figcaption" => {
                    out.push_str("\n*");
                    convert_children(node, out, false, list_depth);
                    out.push_str("*\n");
                }
                "sup" => {
                    out.push_str("^(");
                    convert_children(node, out, false, list_depth);
                    out.push(')');
                }
                "dl" => {
                    out.push('\n');
                    convert_children(node, out, false, list_depth);
                    out.push('\n');
                }
                "dt" => {
                    out.push_str("\n**");
                    convert_children(node, out, false, list_depth);
                    out.push_str("**\n");
                }
                "dd" => {
                    out.push_str(": ");
                    convert_children(node, out, false, list_depth);
                    out.push('\n');
                }
                _ => {
                    convert_children(node, out, in_pre, list_depth);
                }
            }
        }
        NodeData::Document => {
            convert_children(node, out, in_pre, list_depth);
        }
        _ => {}
    }
}

fn convert_children(node: &Handle, out: &mut String, in_pre: bool, list_depth: usize) {
    for child in node.children.borrow().iter() {
        convert_node(child, out, in_pre, list_depth);
    }
}

fn convert_table(node: &Handle, out: &mut String) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut has_header = false;
    collect_table_rows(node, &mut rows, &mut has_header);

    if rows.is_empty() {
        return;
    }

    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        return;
    }

    for row in &mut rows {
        while row.len() < col_count {
            row.push(String::new());
        }
    }

    let widths: Vec<usize> = (0..col_count)
        .map(|c| rows.iter().map(|r| r[c].len()).max().unwrap_or(3).max(3))
        .collect();

    let first = &rows[0];
    out.push('|');
    for (i, cell) in first.iter().enumerate() {
        out.push_str(&format!(" {:width$} |", cell, width = widths[i]));
    }
    out.push('\n');

    out.push('|');
    for w in &widths {
        out.push(' ');
        for _ in 0..*w {
            out.push('-');
        }
        out.push_str(" |");
    }
    out.push('\n');

    for row in rows.iter().skip(1) {
        out.push('|');
        for (i, cell) in row.iter().enumerate() {
            out.push_str(&format!(" {:width$} |", cell, width = widths[i]));
        }
        out.push('\n');
    }
}

fn collect_table_rows(node: &Handle, rows: &mut Vec<Vec<String>>, has_header: &mut bool) {
    if let Some(tag) = tag_name(node) {
        match tag.as_str() {
            "thead" => {
                *has_header = true;
                for child in node.children.borrow().iter() {
                    collect_table_rows(child, rows, has_header);
                }
                return;
            }
            "tr" => {
                let mut cells = Vec::new();
                for child in node.children.borrow().iter() {
                    if let Some(t) = tag_name(child) {
                        if t == "td" || t == "th" {
                            let mut cell_text = String::new();
                            convert_children(child, &mut cell_text, false, 0);
                            cells.push(cell_text.trim().replace('\n', " "));
                        }
                    }
                }
                rows.push(cells);
                return;
            }
            _ => {}
        }
    }
    for child in node.children.borrow().iter() {
        collect_table_rows(child, rows, has_header);
    }
}

fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_ws = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                result.push(' ');
            }
            prev_ws = true;
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }
    result
}

fn clean_output(md: &str) -> String {
    let mut out = String::with_capacity(md.len());
    let mut blank_count = 0;

    for line in md.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(trimmed);
            out.push('\n');
        }
    }

    let result = out.trim().to_string();
    if result.is_empty() {
        result
    } else {
        format!("{result}\n")
    }
}

// Helper: parse HTML string and run full pipeline (parse -> find content -> markdown -> clean)
#[cfg(test)]
fn html_to_md(html: &str) -> String {
    let dom = parse_html(html);
    let root = find_content(&dom.document);
    let md = to_markdown(&root);
    clean_output(&md)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // collapse_whitespace
    // =========================================================================

    #[test]
    fn collapse_whitespace_basic() {
        assert_eq!(collapse_whitespace("hello   world"), "hello world");
    }

    #[test]
    fn collapse_whitespace_tabs_newlines() {
        assert_eq!(collapse_whitespace("hello\t\n\r  world"), "hello world");
    }

    #[test]
    fn collapse_whitespace_leading_trailing() {
        assert_eq!(collapse_whitespace("  hello  "), " hello ");
    }

    #[test]
    fn collapse_whitespace_empty() {
        assert_eq!(collapse_whitespace(""), "");
    }

    #[test]
    fn collapse_whitespace_only_spaces() {
        assert_eq!(collapse_whitespace("     "), " ");
    }

    #[test]
    fn collapse_whitespace_no_change() {
        assert_eq!(collapse_whitespace("hello world"), "hello world");
    }

    #[test]
    fn collapse_whitespace_unicode() {
        // \u{00a0} is non-breaking space, which is_whitespace() returns true for
        assert_eq!(collapse_whitespace("hello\u{00a0}\u{00a0}world"), "hello world");
    }

    // =========================================================================
    // clean_output
    // =========================================================================

    #[test]
    fn clean_output_removes_excess_blanks() {
        let input = "hello\n\n\n\n\nworld";
        let result = clean_output(input);
        assert_eq!(result, "hello\n\n\nworld\n");
    }

    #[test]
    fn clean_output_trims_trailing_spaces() {
        let input = "hello   \nworld   ";
        let result = clean_output(input);
        assert_eq!(result, "hello\nworld\n");
    }

    #[test]
    fn clean_output_empty_input() {
        assert_eq!(clean_output(""), "");
    }

    #[test]
    fn clean_output_only_whitespace() {
        assert_eq!(clean_output("   \n\n  \n  "), "");
    }

    #[test]
    fn clean_output_single_line() {
        assert_eq!(clean_output("hello"), "hello\n");
    }

    #[test]
    fn clean_output_preserves_double_blank() {
        let input = "a\n\nb";
        let result = clean_output(input);
        assert_eq!(result, "a\n\nb\n");
    }

    // =========================================================================
    // should_strip
    // =========================================================================

    #[test]
    fn strip_script() { assert!(should_strip("script")); }
    #[test]
    fn strip_style() { assert!(should_strip("style")); }
    #[test]
    fn strip_nav() { assert!(should_strip("nav")); }
    #[test]
    fn strip_footer() { assert!(should_strip("footer")); }
    #[test]
    fn strip_aside() { assert!(should_strip("aside")); }
    #[test]
    fn strip_iframe() { assert!(should_strip("iframe")); }
    #[test]
    fn strip_svg() { assert!(should_strip("svg")); }
    #[test]
    fn strip_canvas() { assert!(should_strip("canvas")); }
    #[test]
    fn strip_form() { assert!(should_strip("form")); }
    #[test]
    fn strip_noscript() { assert!(should_strip("noscript")); }
    #[test]
    fn strip_template() { assert!(should_strip("template")); }
    #[test]
    fn no_strip_div() { assert!(!should_strip("div")); }
    #[test]
    fn no_strip_article() { assert!(!should_strip("article")); }
    #[test]
    fn no_strip_main() { assert!(!should_strip("main")); }
    #[test]
    fn no_strip_p() { assert!(!should_strip("p")); }

    // =========================================================================
    // Headings
    // =========================================================================

    #[test]
    fn heading_h1() {
        let md = html_to_md("<h1>Title</h1>");
        assert!(md.contains("# Title"));
    }

    #[test]
    fn heading_h2() {
        let md = html_to_md("<h2>Subtitle</h2>");
        assert!(md.contains("## Subtitle"));
    }

    #[test]
    fn heading_h3() {
        let md = html_to_md("<h3>Section</h3>");
        assert!(md.contains("### Section"));
    }

    #[test]
    fn heading_h4() {
        let md = html_to_md("<h4>Sub</h4>");
        assert!(md.contains("#### Sub"));
    }

    #[test]
    fn heading_h5() {
        let md = html_to_md("<h5>Deep</h5>");
        assert!(md.contains("##### Deep"));
    }

    #[test]
    fn heading_h6() {
        let md = html_to_md("<h6>Deepest</h6>");
        assert!(md.contains("###### Deepest"));
    }

    #[test]
    fn heading_with_inline() {
        let md = html_to_md("<h2>Hello <strong>bold</strong> world</h2>");
        assert!(md.contains("## Hello **bold** world"));
    }

    // =========================================================================
    // Paragraphs
    // =========================================================================

    #[test]
    fn paragraph_basic() {
        let md = html_to_md("<p>Hello world</p>");
        assert!(md.contains("Hello world"));
    }

    #[test]
    fn paragraph_multiple() {
        let md = html_to_md("<p>First</p><p>Second</p>");
        assert!(md.contains("First"));
        assert!(md.contains("Second"));
    }

    #[test]
    fn paragraph_whitespace_collapsed() {
        let md = html_to_md("<p>Hello    world   test</p>");
        assert!(md.contains("Hello world test"));
    }

    // =========================================================================
    // Links
    // =========================================================================

    #[test]
    fn link_basic() {
        let md = html_to_md(r#"<a href="https://example.com">Click</a>"#);
        assert!(md.contains("[Click](https://example.com)"));
    }

    #[test]
    fn link_no_href() {
        let md = html_to_md("<a>Orphan</a>");
        assert!(md.contains("[Orphan]()"));
    }

    #[test]
    fn link_with_nested_bold() {
        let md = html_to_md(r#"<a href="/x"><strong>Bold Link</strong></a>"#);
        assert!(md.contains("[**Bold Link**](/x)"));
    }

    #[test]
    fn link_empty_text() {
        let md = html_to_md(r#"<a href="https://example.com"></a>"#);
        assert!(md.contains("[](https://example.com)"));
    }

    // =========================================================================
    // Images
    // =========================================================================

    #[test]
    fn image_basic() {
        let md = html_to_md(r#"<img src="pic.jpg" alt="A photo">"#);
        assert!(md.contains("![A photo](pic.jpg)"));
    }

    #[test]
    fn image_no_alt() {
        let md = html_to_md(r#"<img src="pic.jpg">"#);
        assert!(md.contains("![Image](pic.jpg)"));
    }

    #[test]
    fn image_no_src() {
        let md = html_to_md(r#"<img alt="broken">"#);
        assert!(md.contains("![broken]()"));
    }

    // =========================================================================
    // Inline formatting
    // =========================================================================

    #[test]
    fn strong_tag() {
        let md = html_to_md("<strong>bold</strong>");
        assert!(md.contains("**bold**"));
    }

    #[test]
    fn b_tag() {
        let md = html_to_md("<b>bold</b>");
        assert!(md.contains("**bold**"));
    }

    #[test]
    fn em_tag() {
        let md = html_to_md("<em>italic</em>");
        assert!(md.contains("*italic*"));
    }

    #[test]
    fn i_tag() {
        let md = html_to_md("<i>italic</i>");
        assert!(md.contains("*italic*"));
    }

    #[test]
    fn inline_code() {
        let md = html_to_md("<code>foo()</code>");
        assert!(md.contains("`foo()`"));
    }

    #[test]
    fn nested_bold_italic() {
        let md = html_to_md("<strong><em>both</em></strong>");
        assert!(md.contains("***both***"));
    }

    // =========================================================================
    // Code blocks
    // =========================================================================

    #[test]
    fn pre_code_block() {
        let md = html_to_md("<pre><code>fn main() {}</code></pre>");
        assert!(md.contains("```\nfn main() {}\n```"));
    }

    #[test]
    fn pre_preserves_whitespace() {
        let md = html_to_md("<pre>  indented\n    more</pre>");
        assert!(md.contains("  indented\n    more"));
    }

    #[test]
    fn code_inside_pre_no_backticks() {
        let md = html_to_md("<pre><code>let x = 1;</code></pre>");
        // Should NOT have backticks around the code inside pre
        assert!(!md.contains("`let x = 1;`"));
        assert!(md.contains("let x = 1;"));
    }

    // =========================================================================
    // Blockquotes
    // =========================================================================

    #[test]
    fn blockquote_basic() {
        let md = html_to_md("<blockquote><p>Wise words</p></blockquote>");
        assert!(md.contains("> Wise words"));
    }

    #[test]
    fn blockquote_multiline() {
        let md = html_to_md("<blockquote><p>Line 1</p><p>Line 2</p></blockquote>");
        assert!(md.contains("> "));
    }

    // =========================================================================
    // Lists
    // =========================================================================

    #[test]
    fn unordered_list() {
        let md = html_to_md("<ul><li>One</li><li>Two</li><li>Three</li></ul>");
        assert!(md.contains("- One"));
        assert!(md.contains("- Two"));
        assert!(md.contains("- Three"));
    }

    #[test]
    fn ordered_list() {
        let md = html_to_md("<ol><li>First</li><li>Second</li></ol>");
        assert!(md.contains("- First"));
        assert!(md.contains("- Second"));
    }

    #[test]
    fn nested_list() {
        let md = html_to_md("<ul><li>Top<ul><li>Nested</li></ul></li></ul>");
        assert!(md.contains("- Top"));
        assert!(md.contains("  - Nested"));
    }

    #[test]
    fn deeply_nested_list() {
        let md = html_to_md(
            "<ul><li>A<ul><li>B<ul><li>C</li></ul></li></ul></li></ul>"
        );
        assert!(md.contains("- A"));
        assert!(md.contains("  - B"));
        assert!(md.contains("    - C"));
    }

    #[test]
    fn list_with_links() {
        let md = html_to_md(r#"<ul><li><a href="/x">Link</a></li></ul>"#);
        assert!(md.contains("- [Link](/x)"));
    }

    // =========================================================================
    // Tables
    // =========================================================================

    #[test]
    fn table_basic() {
        let html = r#"
        <table>
            <thead><tr><th>Name</th><th>Age</th></tr></thead>
            <tbody><tr><td>Alice</td><td>30</td></tr></tbody>
        </table>"#;
        let md = html_to_md(html);
        assert!(md.contains("| Name"));
        assert!(md.contains("| Alice"));
        assert!(md.contains("---"));
    }

    #[test]
    fn table_no_thead() {
        let html = "<table><tr><td>A</td><td>B</td></tr><tr><td>C</td><td>D</td></tr></table>";
        let md = html_to_md(html);
        assert!(md.contains("| A"));
        assert!(md.contains("| C"));
    }

    #[test]
    fn table_uneven_columns() {
        let html = "<table><tr><td>A</td><td>B</td><td>C</td></tr><tr><td>D</td></tr></table>";
        let md = html_to_md(html);
        // Second row should be padded
        assert!(md.contains("|"));
    }

    #[test]
    fn table_empty() {
        let md = html_to_md("<table></table>");
        // Should not crash, output may be empty
        assert!(!md.contains("| |"));
    }

    #[test]
    fn table_with_inline_formatting() {
        let html = "<table><tr><td><strong>Bold</strong></td><td><em>Italic</em></td></tr></table>";
        let md = html_to_md(html);
        assert!(md.contains("**Bold**"));
        assert!(md.contains("*Italic*"));
    }

    // =========================================================================
    // Stripping: scripts, styles, nav, etc.
    // =========================================================================

    #[test]
    fn strip_script_tag() {
        let md = html_to_md("<body><p>Keep</p><script>alert('xss')</script></body>");
        assert!(md.contains("Keep"));
        assert!(!md.contains("alert"));
    }

    #[test]
    fn strip_style_tag() {
        let md = html_to_md("<body><p>Keep</p><style>.foo { color: red; }</style></body>");
        assert!(md.contains("Keep"));
        assert!(!md.contains("color"));
    }

    #[test]
    fn strip_nav_html() {
        let md = html_to_md("<body><nav><a href='/'>Home</a></nav><p>Content</p></body>");
        assert!(md.contains("Content"));
        assert!(!md.contains("Home"));
    }

    #[test]
    fn strip_footer_html() {
        let md = html_to_md("<body><p>Content</p><footer>Copyright 2024</footer></body>");
        assert!(md.contains("Content"));
        assert!(!md.contains("Copyright"));
    }

    #[test]
    fn strip_aside_html() {
        let md = html_to_md("<body><p>Main</p><aside>Sidebar</aside></body>");
        assert!(md.contains("Main"));
        assert!(!md.contains("Sidebar"));
    }

    #[test]
    fn strip_iframe_html() {
        let md = html_to_md(r#"<body><p>Text</p><iframe src="ad.html"></iframe></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("ad.html"));
    }

    #[test]
    fn strip_svg_html() {
        let md = html_to_md(r#"<body><p>Text</p><svg><circle r="5"/></svg></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("circle"));
    }

    #[test]
    fn strip_canvas_html() {
        let md = html_to_md("<body><p>Text</p><canvas>Fallback</canvas></body>");
        assert!(md.contains("Text"));
        assert!(!md.contains("Fallback"));
    }

    #[test]
    fn strip_form_html() {
        let md = html_to_md(r#"<body><p>Text</p><form><input type="text"></form></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("input"));
    }

    #[test]
    fn strip_noscript_html() {
        let md = html_to_md("<body><p>Text</p><noscript>Enable JS</noscript></body>");
        assert!(md.contains("Text"));
        assert!(!md.contains("Enable"));
    }

    // =========================================================================
    // Hidden elements
    // =========================================================================

    #[test]
    fn hidden_aria_hidden() {
        let md = html_to_md(r#"<body><p>Visible</p><div aria-hidden="true">Hidden</div></body>"#);
        assert!(md.contains("Visible"));
        assert!(!md.contains("Hidden"));
    }

    #[test]
    fn hidden_attribute() {
        let md = html_to_md(r#"<body><p>Visible</p><div hidden>Secret</div></body>"#);
        assert!(md.contains("Visible"));
        assert!(!md.contains("Secret"));
    }

    #[test]
    fn hidden_display_none() {
        let md = html_to_md(r#"<body><p>Visible</p><div style="display:none">Gone</div></body>"#);
        assert!(md.contains("Visible"));
        assert!(!md.contains("Gone"));
    }

    #[test]
    fn hidden_display_none_spaced() {
        let md = html_to_md(r#"<body><p>Visible</p><div style="display: none">Gone</div></body>"#);
        assert!(md.contains("Visible"));
        assert!(!md.contains("Gone"));
    }

    #[test]
    fn hidden_visibility_hidden() {
        let md = html_to_md(r#"<body><p>Visible</p><div style="visibility:hidden">Ghost</div></body>"#);
        assert!(md.contains("Visible"));
        assert!(!md.contains("Ghost"));
    }

    #[test]
    fn hidden_cookie_class() {
        let md = html_to_md(r#"<body><p>Text</p><div class="cookie-banner">Accept</div></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("Accept"));
    }

    #[test]
    fn hidden_consent_id() {
        let md = html_to_md(r#"<body><p>Text</p><div id="consent-popup">Cookies</div></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("Cookies"));
    }

    #[test]
    fn hidden_modal_class() {
        let md = html_to_md(r#"<body><p>Text</p><div class="modal-overlay">Subscribe</div></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("Subscribe"));
    }

    #[test]
    fn hidden_ad_class() {
        let md = html_to_md(r#"<body><p>Text</p><div class="ad-container">Buy now</div></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("Buy now"));
    }

    #[test]
    fn hidden_tracking_class() {
        let md = html_to_md(r#"<body><p>Text</p><div class="tracking-pixel">1x1</div></body>"#);
        assert!(md.contains("Text"));
        assert!(!md.contains("1x1"));
    }

    #[test]
    fn hidden_role_banner() {
        let md = html_to_md(r#"<body><div role="banner">Banner</div><p>Content</p></body>"#);
        assert!(md.contains("Content"));
        assert!(!md.contains("Banner"));
    }

    #[test]
    fn hidden_role_navigation() {
        let md = html_to_md(r#"<body><div role="navigation">Nav</div><p>Content</p></body>"#);
        assert!(md.contains("Content"));
        assert!(!md.contains("Nav"));
    }

    #[test]
    fn hidden_role_dialog() {
        let md = html_to_md(r#"<body><div role="dialog">Popup</div><p>Content</p></body>"#);
        assert!(md.contains("Content"));
        assert!(!md.contains("Popup"));
    }

    // =========================================================================
    // Content priority: article > main > body
    // =========================================================================

    #[test]
    fn prefers_article() {
        let html = r#"<body><div>Body junk</div><article><p>Article content</p></article></body>"#;
        let md = html_to_md(html);
        assert!(md.contains("Article content"));
        assert!(!md.contains("Body junk"));
    }

    #[test]
    fn prefers_main_over_body() {
        let html = r#"<body><div>Body junk</div><main><p>Main content</p></main></body>"#;
        let md = html_to_md(html);
        assert!(md.contains("Main content"));
        assert!(!md.contains("Body junk"));
    }

    #[test]
    fn article_over_main() {
        let html = r#"<body><main><p>Main</p></main><article><p>Article</p></article></body>"#;
        let md = html_to_md(html);
        assert!(md.contains("Article"));
        // Should be scoped to article, not main
    }

    #[test]
    fn falls_back_to_body() {
        let html = "<body><p>Just body</p></body>";
        let md = html_to_md(html);
        assert!(md.contains("Just body"));
    }

    // =========================================================================
    // Figure / figcaption
    // =========================================================================

    #[test]
    fn figure_with_caption() {
        let html = r#"<figure><img src="x.jpg" alt="Photo"><figcaption>A caption</figcaption></figure>"#;
        let md = html_to_md(html);
        assert!(md.contains("![Photo](x.jpg)"));
        assert!(md.contains("*A caption*"));
    }

    // =========================================================================
    // Sup
    // =========================================================================

    #[test]
    fn superscript() {
        let md = html_to_md("<p>E=mc<sup>2</sup></p>");
        assert!(md.contains("E=mc^(2)"));
    }

    // =========================================================================
    // BR / HR
    // =========================================================================

    #[test]
    fn br_tag() {
        let md = html_to_md("<p>Line1<br>Line2</p>");
        assert!(md.contains("Line1\nLine2"));
    }

    #[test]
    fn hr_tag() {
        let md = html_to_md("<p>Above</p><hr><p>Below</p>");
        assert!(md.contains("---"));
    }

    // =========================================================================
    // Definition lists
    // =========================================================================

    #[test]
    fn definition_list() {
        let html = "<dl><dt>Term</dt><dd>Definition</dd></dl>";
        let md = html_to_md(html);
        assert!(md.contains("**Term**"));
        assert!(md.contains(": Definition"));
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn empty_html() {
        let md = html_to_md("");
        assert!(md.is_empty() || md.trim().is_empty());
    }

    #[test]
    fn only_whitespace_html() {
        let md = html_to_md("   \n\n   ");
        assert!(md.trim().is_empty());
    }

    #[test]
    fn deeply_nested_divs() {
        let html = "<div><div><div><div><p>Deep content</p></div></div></div></div>";
        let md = html_to_md(html);
        assert!(md.contains("Deep content"));
    }

    #[test]
    fn mixed_content() {
        let html = r#"
        <body>
            <nav><a href="/">Home</a></nav>
            <article>
                <h1>Title</h1>
                <p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>
                <ul><li>Item 1</li><li>Item 2</li></ul>
                <blockquote><p>A quote</p></blockquote>
                <pre><code>code();</code></pre>
            </article>
            <footer>Footer text</footer>
        </body>"#;
        let md = html_to_md(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("**bold**"));
        assert!(md.contains("*italic*"));
        assert!(md.contains("- Item 1"));
        assert!(md.contains("> A quote"));
        assert!(md.contains("```\ncode();\n```"));
        assert!(!md.contains("Home"));     // nav stripped
        assert!(!md.contains("Footer"));   // footer stripped
    }

    #[test]
    fn special_characters_in_text() {
        let md = html_to_md("<p>5 > 3 &amp; 2 < 4</p>");
        assert!(md.contains("5 > 3 & 2 < 4"));
    }

    #[test]
    fn entities_decoded() {
        let md = html_to_md("<p>&quot;hello&quot; &mdash; world</p>");
        assert!(md.contains("\"hello\""));
    }

    #[test]
    fn multiple_scripts_stripped() {
        let html = r#"<body>
            <script>var a=1;</script>
            <p>Keep</p>
            <script>var b=2;</script>
            <script type="application/ld+json">{"@type":"Article"}</script>
        </body>"#;
        let md = html_to_md(html);
        assert!(md.contains("Keep"));
        assert!(!md.contains("var a"));
        assert!(!md.contains("var b"));
        assert!(!md.contains("Article"));
    }

    #[test]
    fn inline_style_stripped() {
        let html = r#"<body><style>.x{color:red}</style><p>Content</p></body>"#;
        let md = html_to_md(html);
        assert!(md.contains("Content"));
        assert!(!md.contains("color"));
    }

    #[test]
    fn link_inside_heading() {
        let md = html_to_md(r#"<h2><a href="/about">About Us</a></h2>"#);
        assert!(md.contains("## [About Us](/about)"));
    }

    #[test]
    fn image_inside_link() {
        let md = html_to_md(r#"<a href="/page"><img src="thumb.jpg" alt="Thumb"></a>"#);
        assert!(md.contains("[![Thumb](thumb.jpg)](/page)"));
    }

    #[test]
    fn empty_paragraph() {
        let md = html_to_md("<p></p><p>Real content</p>");
        assert!(md.contains("Real content"));
    }

    #[test]
    fn whitespace_only_paragraph() {
        let md = html_to_md("<p>   </p><p>Real</p>");
        assert!(md.contains("Real"));
    }

    #[test]
    fn adjacent_inline_elements() {
        let md = html_to_md("<p><strong>bold</strong><em>italic</em></p>");
        assert!(md.contains("**bold***italic*"));
    }

    #[test]
    fn table_single_cell() {
        let md = html_to_md("<table><tr><td>Alone</td></tr></table>");
        assert!(md.contains("| Alone"));
    }

    #[test]
    fn table_with_links() {
        let html = r#"<table><tr><td><a href="/x">Link</a></td><td>Text</td></tr></table>"#;
        let md = html_to_md(html);
        assert!(md.contains("[Link](/x)"));
        assert!(md.contains("Text"));
    }

    #[test]
    fn list_with_paragraph_inside() {
        let html = "<ul><li><p>Para in li</p></li></ul>";
        let md = html_to_md(html);
        // <p> inside <li> causes block-level break; content still present
        assert!(md.contains("-"));
        assert!(md.contains("Para in li"));
    }

    #[test]
    fn multiple_tables() {
        let html = r#"
        <table><tr><td>T1</td></tr></table>
        <table><tr><td>T2</td></tr></table>"#;
        let md = html_to_md(html);
        assert!(md.contains("T1"));
        assert!(md.contains("T2"));
    }

    #[test]
    fn code_with_html_inside() {
        let md = html_to_md("<code>&lt;div&gt;</code>");
        assert!(md.contains("`<div>`"));
    }

    #[test]
    fn pre_with_newlines() {
        let html = "<pre>line1\nline2\nline3</pre>";
        let md = html_to_md(html);
        assert!(md.contains("line1\nline2\nline3"));
    }

    #[test]
    fn blockquote_with_formatting() {
        let html = "<blockquote><p><strong>Bold</strong> quote</p></blockquote>";
        let md = html_to_md(html);
        assert!(md.contains("> **Bold** quote"));
    }

    #[test]
    fn nested_blockquotes() {
        let html = "<blockquote><p>Outer</p><blockquote><p>Inner</p></blockquote></blockquote>";
        let md = html_to_md(html);
        assert!(md.contains("> "));
        assert!(md.contains("Outer"));
        assert!(md.contains("Inner"));
    }

    // =========================================================================
    // Large / realistic HTML structures
    // =========================================================================

    #[test]
    fn full_page_structure() {
        let html = r#"<!DOCTYPE html>
        <html>
        <head><title>Test Page</title><style>body{margin:0}</style></head>
        <body>
            <header><nav><a href="/">Home</a><a href="/about">About</a></nav></header>
            <main>
                <article>
                    <h1>Article Title</h1>
                    <p>First paragraph with <a href="/link">a link</a>.</p>
                    <h2>Section One</h2>
                    <p>Some text with <code>inline code</code>.</p>
                    <pre><code>fn example() {
    println!("hello");
}</code></pre>
                    <h2>Section Two</h2>
                    <ul>
                        <li>Item A</li>
                        <li>Item B with <strong>bold</strong></li>
                    </ul>
                    <table>
                        <thead><tr><th>Col1</th><th>Col2</th></tr></thead>
                        <tbody><tr><td>Val1</td><td>Val2</td></tr></tbody>
                    </table>
                </article>
            </main>
            <aside><p>Related stuff</p></aside>
            <footer><p>Copyright 2024</p></footer>
            <script>console.log("tracked")</script>
        </body>
        </html>"#;
        let md = html_to_md(html);

        // Content present
        assert!(md.contains("# Article Title"));
        assert!(md.contains("## Section One"));
        assert!(md.contains("## Section Two"));
        assert!(md.contains("[a link](/link)"));
        assert!(md.contains("`inline code`"));
        assert!(md.contains("```\nfn example()"));
        assert!(md.contains("- Item A"));
        assert!(md.contains("**bold**"));
        assert!(md.contains("| Col1"));
        assert!(md.contains("| Val1"));

        // Junk stripped
        assert!(!md.contains("Home"));
        assert!(!md.contains("About"));
        assert!(!md.contains("Related stuff"));
        assert!(!md.contains("Copyright"));
        assert!(!md.contains("console.log"));
        assert!(!md.contains("margin:0"));
    }

    #[test]
    fn no_double_backtick_in_pre() {
        // pre>code should NOT produce ```\n`code`\n```
        let html = "<pre><code>x = 1</code></pre>";
        let md = html_to_md(html);
        let count = md.matches('`').count();
        // Should have exactly 6 backticks: ``` opener + ``` closer
        assert_eq!(count, 6, "Expected exactly 6 backticks, got {count} in: {md}");
    }
}
