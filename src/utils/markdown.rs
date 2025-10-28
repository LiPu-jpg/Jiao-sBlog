use pulldown_cmark::{Parser, Options, html};

pub fn render_markdown(md_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    let parser = Parser::new_ext(md_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
