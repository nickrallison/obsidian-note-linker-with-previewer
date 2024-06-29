use core::panic;
use std::{
    path::{Path, PathBuf},
    vec,
};

use pest::Parser;
use pest_derive::Parser;

/*
Grammar:
#####
string_char = _{ (ASCII_ALPHANUMERIC | (!('\u{00}'..'\u{7F}') ~ ANY) | "-" | "–" | "_" | "'" | "\"" | "\\*" | "\\[" | "\\]" | " " | "\t" | "," | "." | "!" | "?" | "(" | ")" | "+" | "=" | ";" | ":" | "/" | "%" | "^" | "{" | "}" | "|" | "\\" | ">" | "<" ) }

filepath = { (!"$" ~ !"*" ~ !"\n" ~ !"[" ~ !"]" ~ !">" ~ ANY)+ }

weblink_link = { (!")" ~ ANY)+ }
weblink_text = { string_char+ }

bold_italic_node = { "*"{3} ~ ( named_link_node | latex_inline_node | link_node | weblink_node | node)+ ~ "*"{3} }
bold_node = { "*"{2} ~ ( named_link_node | latex_inline_node | link_node | weblink_node | node)+ ~ "*"{2} }
italic_node = { "*" ~ ( named_link_node | latex_inline_node | link_node | weblink_node | node)+ ~ "*" }
named_link_node = { "["{2} ~ filepath ~ "|" ~ node+ ~ "]"{2} }
link_node = { "["{2} ~ filepath ~ "]"{2} }
weblink_node = { "[" ~ weblink_text ~ "]" ~ "(" ~ weblink_link ~ ")"}
square_bracket_node = { "[" ~ (!"]" ~ ANY)+ ~ "]" }
latex_inline_node = { "$" ~ (!"$" ~ ANY)+ ~ "$" }
code_inline_node = { "`" ~ (!"`" ~ ANY)+ ~ "`" }
node = { string_char+ }

heading_line = { "#"{1,6} ~ " " ~ string_line }
numbered_list_line = { (" " | "\r" | "\t")* ~ ASCII_DIGIT+ ~ "." ~ (" " | "\r" | "\t")+ ~ string_line}
list_line = { (" " | "\r" | "\t")* ~ "-" ~ (" " | "\r" | "\t")* ~ string_line}
string_line = { (bold_italic_node | bold_node | italic_node | named_link_node | link_node | weblink_node | square_bracket_node | latex_inline_node | code_inline_node | node)* }

line = { (heading_line | numbered_list_line | list_line | string_line ) }
block_quote_line = { ((" " | "\r" | "\t")* ~ ">" ~ (block_quote_line | line)) }

code_type = { (ASCII_ALPHANUMERIC | "_" | "-" )+ }
code_block_inner = { (!"```" ~ ANY)* }

block_quote_block = { block_quote_line+ }
latex_block = { "$$" ~ (!"$" ~ ANY)* ~ "$$" }
code_block = { "```" ~ code_type? ~ code_block_inner ~ "```" }
string_block = { ( line ~ NEWLINE )+ }


block = { (block_quote_block | latex_block | code_block | string_block) }

yaml_inner = { (!"---" ~ ANY)* }
yaml = { "---" ~ yaml_inner ~ "---" }

md_file = { SOI ~ yaml? ~ block+ ~ EOI }
*/

use crate::prelude::*;

#[derive(Parser)]
#[grammar = "src/rust/parser/md.pest"]
pub struct MDParser;

pub fn parse_md_file_wrapper(contents: String, path: String) -> Result<MDFile> {
    let path = PathBuf::from(path);
    let mut contents = contents;
    if !&contents.ends_with('\n') {
        contents.push('\n');
    }

    let parse_result = MDParser::parse(Rule::md_file, &contents);
    let mut md_file = match parse_result {
        Ok(md_file) => md_file,
        Err(e) => {
            return Err(Error::ParseError(
                path.to_path_buf(),
                format!("Error: {}", e),
            ))
        }
    };
    let pairs_result = md_file.next();
    let pairs = match pairs_result {
        Some(pairs) => pairs,
        None => {
            return Err(Error::ParseError(
                path.to_path_buf(),
                format!("No parse result"),
            ))
        }
    };

    let mut md_file_struct: MDFile = parse_md_file(pairs, &path)?;
    md_file_struct.path = path;
    Ok(md_file_struct)
}

#[derive(Debug)]
pub struct MDFile {
    pub yaml: Option<YAML>,
    pub blocks: Vec<Block>,

    pub path: PathBuf, // absolute path to the file
}

impl MDFile {
    pub fn get_yaml(&self) -> Option<&serde_yaml::Value> {
        self.yaml.as_ref().map(|yaml| &yaml.yaml)
    }

    // pub fn get_blocks(&self) -> &Vec<Block> {
    //     &self.blocks
    // }

    pub fn get_title(&self) -> &str {
        // basename of the file
        self.path.file_stem().unwrap().to_str().unwrap()
    }

    pub fn get_aliases(&self) -> Result<Vec<&str>> {
        let yaml: &serde_yaml::Value = match self.yaml.as_ref() {
            Some(yaml) => &yaml.yaml,
            None => {
                return Err(Error::Generic(format!(
                    "No yaml for file: {}",
                    self.path.display()
                )))
            }
        };

        let aliases: Option<&Vec<serde_yaml::Value>> = yaml["aliases"].as_sequence();
        let aliases: &Vec<serde_yaml::Value> = match aliases {
            Some(aliases) => aliases,
            None => {
                return Err(Error::Generic(format!(
                    "No aliases for file: {}",
                    self.path.display()
                )))
            }
        };
        for alias in aliases {
            let alias: &serde_yaml::Value = alias;
            let alias: &str = match alias.as_str() {
                Some(alias) => alias,
                None => {
                    return Err(Error::Generic(format!(
                        "Alias: {:?} is not a string for file: {}",
                        alias,
                        self.path.display()
                    )))
                }
            };
        }
        let aliases: Vec<&str> = aliases
            .iter()
            .map(|alias| {
                alias
                    .as_str()
                    .expect("Non strings should have been caught above")
            })
            .collect();
        Ok(aliases)
    }

    pub fn get_string_nodes(&self) -> Vec<StringPosition> {
        let mut nodes: Vec<StringPosition> = Vec::new();
        for block in &self.blocks {
            for node in block.get_string_nodes() {
                nodes.push(node);
            }
        }
        nodes
    }
}

fn parse_md_file(pairs: pest::iterators::Pair<Rule>, path: &Path) -> Result<MDFile> {
    debug_assert!(pairs.as_rule() == Rule::md_file);
    let mut result: MDFile = MDFile {
        yaml: None,
        blocks: Vec::new(),
        path: Default::default(),
    };

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::yaml => {
                result.yaml = Some(parse_yaml(pair, &path)?);
            }
            Rule::block => {
                result.blocks.push(parse_block(pair, &path)?);
            }
            Rule::EOI => {}
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair.as_rule()),
                ))
            }
        }
    }
    Ok(result)
}

#[derive(Debug)]

pub struct YAML {
    pub yaml: serde_yaml::Value,
}

fn parse_yaml(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<YAML> {
    debug_assert!(pair.as_rule() == Rule::yaml);

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::yaml_inner => {
                return Ok(YAML {
                    yaml: serde_yaml::from_str(pair_inner.as_str()).unwrap(),
                });
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner.as_rule()),
                ))
            }
        }
    }
    return Err(Error::ParseError(
        path.to_path_buf(),
        format!("pairs inner is empty"),
    ));
}

#[derive(Debug)]
pub enum Block {
    BlockQuote(BlockQuote),
    Latex(LatexBlock),
    Code(CodeBlock),
    String(StringBlock),
}

impl Block {
    pub fn get_string_nodes(&self) -> Vec<StringPosition> {
        let mut nodes: Vec<StringPosition> = Vec::new();
        match self {
            Block::BlockQuote(block_quote) => {
                for block in &block_quote.inner_blocks {
                    for node in block.get_string_nodes() {
                        nodes.push(node);
                    }
                }
            }
            Block::Latex(latex_block) => {}
            Block::Code(code_block) => {}
            Block::String(string_block) => {
                for line in &string_block.lines {
                    for node in &line.get_string_nodes() {
                        nodes.push(node.clone());
                    }
                }
            }
        }
        nodes
    }
}

fn parse_block(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<Block> {
    debug_assert!(pair.as_rule() == Rule::block || pair.as_rule() == Rule::yaml);

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::block_quote_block => {
                return Ok(Block::BlockQuote(parse_block_quote_block(
                    pair_inner, &path,
                )?));
            }
            Rule::latex_block => {
                return Ok(Block::Latex(parse_latex_block(pair_inner, &path)?));
            }
            Rule::code_block => {
                return Ok(Block::Code(parse_code_block(pair_inner, &path)?));
            }
            Rule::string_block => {
                return Ok(Block::String(parse_string_block(pair_inner, &path)?));
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner.as_rule()),
                ))
            }
        }
    }
    return Err(Error::ParseError(
        path.to_path_buf(),
        format!("pairs inner is empty"),
    ));
}

fn parse_vec_line_into_block(
    pairs: Vec<pest::iterators::Pair<Rule>>,
    path: &Path,
) -> Result<StringBlock> {
    for pair in &pairs {
        debug_assert!(pair.as_rule() == Rule::block || pair.as_rule() == Rule::yaml);
    }
    let mut lines: Vec<Line> = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::line => {
                lines.push(parse_line(pair, &path)?);
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair.as_rule()),
                ))
            }
        }
    }
    Ok(StringBlock { lines })
}

// not including >
#[derive(Debug)]
pub struct BlockQuote {
    pub inner_blocks: Vec<Block>,
}

fn parse_block_quote_block(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<BlockQuote> {
    debug_assert!(pair.as_rule() == Rule::block_quote_block);

    let mut inner_blocks: Vec<Block> = Vec::new();
    let lines: Vec<pest::iterators::Pair<Rule>> = pair.into_inner().collect();
    parse_block_quote_lines(lines, &path);

    Ok(BlockQuote { inner_blocks })
}

#[derive(Debug)]

enum BlockQuoteLineState {
    Start,
    Line,
    BlockQuote,
}

fn parse_block_quote_lines(
    pairs: Vec<pest::iterators::Pair<Rule>>,
    path: &Path,
) -> Result<BlockQuote> {
    for pair in &pairs {
        debug_assert!(pair.as_rule() == Rule::block_quote_line);
    }
    let mut inner_blocks: Vec<Block> = Vec::new();
    let mut state = BlockQuoteLineState::Start;

    let mut current_block: Vec<pest::iterators::Pair<Rule>> = Vec::new();

    for pair in pairs {
        for pair_inner in pair.clone().into_inner() {
            match state {
                BlockQuoteLineState::Start => match pair_inner.as_rule() {
                    Rule::block_quote_line => {
                        current_block.push(pair_inner);
                        state = BlockQuoteLineState::BlockQuote;
                    }
                    Rule::line => {
                        current_block.push(pair_inner);
                        state = BlockQuoteLineState::Line;
                    }
                    _ => {
                        return Err(Error::ParseError(
                            path.to_path_buf(),
                            format!("unexpected rule: {:?}", pair_inner),
                        ))
                    }
                },
                BlockQuoteLineState::Line => match pair_inner.as_rule() {
                    Rule::block_quote_line => {
                        inner_blocks.push(Block::String(parse_vec_line(current_block, &path)?));
                        current_block = Vec::new();
                        current_block.push(pair_inner);
                        state = BlockQuoteLineState::BlockQuote;
                    }
                    Rule::line => {
                        current_block.push(pair_inner);
                    }
                    _ => {
                        return Err(Error::ParseError(
                            path.to_path_buf(),
                            format!("unexpected rule: {:?}", pair_inner),
                        ))
                    }
                },
                BlockQuoteLineState::BlockQuote => match pair_inner.as_rule() {
                    Rule::block_quote_line => {
                        current_block.push(pair_inner);
                    }
                    Rule::line => {
                        let block_quote = parse_block_quote_lines(current_block, &path)?;
                        inner_blocks.push(Block::BlockQuote(block_quote));
                        current_block = Vec::new();
                        current_block.push(pair_inner);
                        state = BlockQuoteLineState::Line;
                    }
                    _ => {
                        return Err(Error::ParseError(
                            path.to_path_buf(),
                            format!("unexpected rule: {:?}", pair_inner),
                        ))
                    }
                },
            }
        }
    }
    match state {
        BlockQuoteLineState::Start => {}
        BlockQuoteLineState::Line => {
            if !current_block.is_empty() {
                inner_blocks.push(Block::String(parse_vec_line(current_block, &path)?));
            }
        }
        BlockQuoteLineState::BlockQuote => {
            if !current_block.is_empty() {
                let block_quote = parse_block_quote_lines(current_block, &path)?;
                inner_blocks.push(Block::BlockQuote(block_quote));
            }
        }
    }

    Ok(BlockQuote { inner_blocks })
}

fn parse_vec_line(pairs: Vec<pest::iterators::Pair<Rule>>, path: &Path) -> Result<StringBlock> {
    for pair in &pairs {
        debug_assert!(pair.as_rule() == Rule::line);
    }
    let mut lines: Vec<Line> = Vec::new();
    for pair in pairs {
        lines.push(parse_line(pair, &path)?);
    }
    Ok(StringBlock { lines })
}

// not including $$
#[derive(Debug)]
pub struct LatexBlock {
    pub latex: String,
}

fn parse_latex_block(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<LatexBlock> {
    debug_assert!(pair.as_rule() == Rule::latex_block);

    let latex = pair.as_str();

    Ok(LatexBlock {
        latex: latex.to_string(),
    })
}

// not including ```
#[derive(Debug)]
pub struct CodeBlock {
    pub code_type: Option<String>,
    pub code: String,
}

fn parse_code_block(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<CodeBlock> {
    debug_assert!(pair.as_rule() == Rule::code_block);

    let mut code_type: Option<String> = None;
    let mut code: String = String::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::code_type => {
                code_type = Some(pair_inner.as_str().to_string());
            }
            Rule::code_block_inner => {
                code = pair_inner.as_str().to_string();
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(CodeBlock { code_type, code })
}
#[derive(Debug)]

pub struct StringBlock {
    pub lines: Vec<Line>,
}

fn parse_string_block(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<StringBlock> {
    debug_assert!(pair.as_rule() == Rule::string_block);

    let mut lines: Vec<Line> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::line => {
                lines.push(parse_line(pair_inner, &path)?);
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(StringBlock { lines })
}

#[derive(Debug)]
pub enum Line {
    NumberedList(NumberedList),
    BulletedList(BulletedList),
    Heading(Heading),
    StringLine(StringLine),
}

impl Line {
    pub fn get_string_nodes(&self) -> Vec<StringPosition> {
        match self {
            Line::NumberedList(numbered_list) => {
                let mut nodes: Vec<StringPosition> = Vec::new();
                for node in &numbered_list.nodes {
                    let inner_nodes = node.get_string_node();
                    for node in inner_nodes {
                        nodes.push(node);
                    }
                }
                nodes
            }
            Line::BulletedList(bulleted_list) => {
                let mut nodes: Vec<StringPosition> = Vec::new();
                for node in &bulleted_list.nodes {
                    let inner_nodes = node.get_string_node();
                    for node in inner_nodes {
                        nodes.push(node);
                    }
                }
                nodes
            }
            Line::Heading(heading) => {
                let mut nodes: Vec<StringPosition> = Vec::new();
                for node in &heading.nodes {
                    let inner_nodes = node.get_string_node();
                    for node in inner_nodes {
                        nodes.push(node);
                    }
                }
                nodes
            }
            Line::StringLine(string_line) => {
                let mut nodes: Vec<StringPosition> = Vec::new();
                for node in &string_line.nodes {
                    let inner_nodes = node.get_string_node();
                    for node in inner_nodes {
                        nodes.push(node);
                    }
                }
                nodes
            }
        }
    }
}

fn parse_line(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<Line> {
    debug_assert!(pair.as_rule() == Rule::line);

    let mut result: Line = Line::StringLine(StringLine { nodes: Vec::new() });

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::heading_line => {
                result = Line::Heading(parse_heading_line(pair_inner, &path)?);
            }
            Rule::numbered_list_line => {
                result = Line::NumberedList(parse_numbered_list_line(pair_inner, &path)?);
            }
            Rule::list_line => {
                result = Line::BulletedList(parse_list_line(pair_inner, &path)?);
            }
            Rule::string_line => {
                result = Line::StringLine(parse_string_line(pair_inner, &path)?);
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(result)
}

#[derive(Debug)]

pub struct NumberedList {
    pub indent: String,
    pub number: u32,
    pub nodes: Vec<Node>,
}

fn parse_numbered_list_line(
    pair: pest::iterators::Pair<Rule>,
    path: &Path,
) -> Result<NumberedList> {
    debug_assert!(pair.as_rule() == Rule::numbered_list_line);

    let mut indent: String = String::new();
    let mut number: u32 = 0;
    let mut nodes: Vec<Node> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::string_line => {
                nodes = parse_string_line(pair_inner, &path)?.nodes;
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(NumberedList {
        indent,
        number,
        nodes,
    })
}

#[derive(Debug)]

pub struct BulletedList {
    pub indent: String,
    pub nodes: Vec<Node>,
}

fn parse_list_line(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<BulletedList> {
    debug_assert!(pair.as_rule() == Rule::list_line);

    let mut indent: String = String::new();
    let mut nodes: Vec<Node> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::string_line => {
                nodes = parse_string_line(pair_inner, &path)?.nodes;
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(BulletedList { indent, nodes })
}

#[derive(Debug)]

pub struct Heading {
    pub level: u32,
    pub nodes: Vec<Node>,
}

fn parse_heading_line(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<Heading> {
    debug_assert!(pair.as_rule() == Rule::heading_line);

    let mut level: u32 = 0;
    let mut nodes: Vec<Node> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::string_line => {
                nodes = parse_string_line(pair_inner, &path)?.nodes;
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(Heading { level, nodes })
}

#[derive(Debug)]

pub struct StringLine {
    pub nodes: Vec<Node>,
}

fn parse_string_line(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<StringLine> {
    debug_assert!(pair.as_rule() == Rule::string_line);

    let mut nodes: Vec<Node> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::bold_italic_node => {
                nodes.push(Node::BoldItalic(pair_inner.as_str().to_string()));
            }
            Rule::bold_node => {
                nodes.push(Node::Bold(pair_inner.as_str().to_string()));
            }
            Rule::italic_node => {
                nodes.push(Node::Italic(pair_inner.as_str().to_string()));
            }
            Rule::named_link_node => {
                nodes.push(Node::NamedMDLink(parse_named_link_node(pair_inner, &path)?));
            }
            Rule::link_node => {
                nodes.push(Node::MDLink(pair_inner.as_str().to_string()));
            }
            Rule::weblink_node => {
                nodes.push(Node::WebLink(parse_weblink_node(pair_inner, &path)?));
            }
            Rule::square_bracket_node => {
                nodes.push(Node::SquareBracket(pair_inner.as_str().to_string()));
            }
            Rule::latex_block_inline_node => {
                nodes.push(Node::InlineLatexBlock(pair_inner.as_str().to_string()));
            }
            Rule::code_block_inline_node => {
                nodes.push(Node::InlineCodeBlock(pair_inner.as_str().to_string()));
            }
            Rule::latex_inline_node => {
                nodes.push(Node::InlineLatex(pair_inner.as_str().to_string()));
            }
            Rule::code_inline_node => {
                nodes.push(Node::InlineCode(pair_inner.as_str().to_string()));
            }
            Rule::node => {
                nodes.push(Node::Text(pair_inner.as_str().to_string()));
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner.as_rule()),
                ))
            }
        }
    }

    Ok(StringLine { nodes })
}

#[derive(Debug)]

pub enum Node {
    Text(String),
    BoldItalic(String),
    Bold(String),
    Italic(String),
    MDLink(String),
    NamedMDLink(NamedMDLink),
    WebLink(WebLink),
    SquareBracket(String),
    InlineCode(String),
    InlineLatex(String),
    InlineCodeBlock(String),
    InlineLatexBlock(String),
}

impl Node {
    pub fn get_string_node(&self) -> Vec<StringPosition> {
        match self {
            Node::Text(_) => {
                vec![StringPosition {
                    string_node: self,
                    line: 0,
                    column: 0,
                }]
            }
            Node::BoldItalic(_) => {
                vec![StringPosition {
                    string_node: self,
                    line: 0,
                    column: 0,
                }]
            }
            Node::Bold(_) => {
                vec![StringPosition {
                    string_node: self,
                    line: 0,
                    column: 0,
                }]
            }
            Node::Italic(_) => {
                vec![StringPosition {
                    string_node: self,
                    line: 0,
                    column: 0,
                }]
            }
            Node::MDLink(_) => vec![],
            Node::NamedMDLink(_) => vec![],
            Node::WebLink(_) => vec![],
            Node::SquareBracket(_) => vec![],
            Node::InlineCode(_) => vec![],
            Node::InlineLatex(_) => vec![],
            Node::InlineCodeBlock(_) => vec![],
            Node::InlineLatexBlock(_) => vec![],
        }
    }

    pub(crate) fn get_inner_string(&self) -> Result<&str> {
        match self {
            Node::Text(s) => Ok(s.as_str()),
            Node::BoldItalic(s) => Ok(s.as_str()),
            Node::Bold(s) => Ok(s.as_str()),
            Node::Italic(s) => Ok(s.as_str()),
            Node::MDLink(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from MDLink"
            ))),
            Node::NamedMDLink(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from NamedMDLink"
            ))),
            Node::WebLink(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from WebLink"
            ))),
            Node::SquareBracket(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from SquareBracket"
            ))),
            Node::InlineCode(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from InlineCode"
            ))),
            Node::InlineLatex(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from InlineLatex"
            ))),
            Node::InlineCodeBlock(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from InlineCodeBlock"
            ))),
            Node::InlineLatexBlock(_) => Err(Error::Generic(f!(
                "Unexpected call to get_inner_string from InlineLatexBlock"
            ))),
        }
    }
}

#[derive(Debug)]

pub struct NamedMDLink {
    pub name: String,
    pub link: String,
}

fn parse_named_link_node(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<NamedMDLink> {
    debug_assert!(pair.as_rule() == Rule::named_link_node);

    let mut name: String = String::new();
    let mut link: String = String::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::filepath => {
                link = pair_inner.as_str().to_string();
            }
            Rule::node => {
                name = pair_inner.as_str().to_string();
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(NamedMDLink { name, link })
}

#[derive(Debug)]

pub struct WebLink {
    pub name: String,
    pub link: String,
}

fn parse_weblink_node(pair: pest::iterators::Pair<Rule>, path: &Path) -> Result<WebLink> {
    debug_assert!(pair.as_rule() == Rule::weblink_node);

    let mut name: String = String::new();
    let mut link: String = String::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::weblink_text => {
                name = pair_inner.as_str().to_string();
            }
            Rule::weblink_link => {
                link = pair_inner.as_str().to_string();
            }
            _ => {
                return Err(Error::ParseError(
                    path.to_path_buf(),
                    format!("unexpected rule: {:?}", pair_inner),
                ))
            }
        }
    }

    Ok(WebLink { name, link })
}

#[derive(Debug, Clone)]
pub(crate) struct StringPosition<'a> {
    pub string_node: &'a Node,
    pub line: u32,
    pub column: u32,
}
