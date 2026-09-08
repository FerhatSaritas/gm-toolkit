//! Markdown → HTML rendering (comrak, GFM). Raw HTML in user files is
//! escaped (comrak's `unsafe` stays off) so rendered output is inert.

use comrak::markdown_to_html;

fn options() -> comrak::Options<'static> {
    let mut opts = comrak::Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.render.r#unsafe = false;
    opts
}

pub fn render(md: &str) -> String {
    markdown_to_html(md, &options())
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn renders_basics() {
        let html = render("# Title\n\nSome *text* with `code`.\n");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<em>text</em>"));
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn renders_tables_and_strikethrough() {
        let html = render("| a | b |\n| - | - |\n| 1 | 2 |\n\n~~gone~~\n");
        assert!(html.contains("<table>"));
        assert!(html.contains("<del>gone</del>"));
    }

    #[test]
    fn renders_blockquotes_for_read_aloud() {
        let html = render("> The fire crackles.\n");
        assert!(html.contains("<blockquote>"));
    }

    #[test]
    fn drops_raw_html() {
        // With `r#unsafe = false`, comrak omits raw HTML entirely —
        // it neither executes nor echoes user HTML.
        let html = render("<script>alert(1)</script>\n\n<b>bold</b> text\n");
        assert!(!html.contains("<script"));
        assert!(!html.contains("<b>"));
        assert!(html.contains("text"));
    }

    #[test]
    fn renders_hr_and_tasklists() {
        let html = render("---\n\n- [x] done\n- [ ] todo\n");
        assert!(html.contains("<hr"));
        assert!(html.contains("task-list") || html.contains("checkbox"));
    }
}
