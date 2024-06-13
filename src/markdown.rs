//! `markdown` handles the parsing of markdown to HTML

#[derive(Debug)]
pub struct Markdown {
    pub blocks: Vec<Vec<String>>,
}
impl Markdown {
    pub fn new(input: String) -> Self {
        let paragraphs: Vec<String> = input.lines().map(|e| e.to_string()).collect();
        Self { blocks: vec![paragraphs] }
    }
}