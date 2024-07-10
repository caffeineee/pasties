use pulldown_cmark::{html, Parser};

pub fn render_markdown(markdown: String) -> String {
    let parser = Parser::new(&markdown);
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    html_buf
}
