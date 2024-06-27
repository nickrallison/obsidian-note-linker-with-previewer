use core::panic;
use std::path::PathBuf;

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

#[derive(Parser)]
#[grammar = "src/parser/md.pest"]
pub struct MDParser;

pub fn parse_md_file_wrapper(contents: String) -> MDFile {
    let mut contents = contents;
    if !&contents.ends_with('\n') {
        contents.push('\n');
    }

    let pairs: pest::iterators::Pair<Rule> = MDParser::parse(Rule::md_file, &contents)
        .expect("unsuccessful parse")
        .next()
        .expect("no parse result");
    let md_file: MDFile = parse_md_file(pairs);

    md_file
}

#[derive(Debug)]
pub struct MDFile {
    yaml: Option<YAML>,
    blocks: Vec<Block>,

    path: PathBuf, // absolute path to the file
}

fn parse_md_file(pairs: pest::iterators::Pair<Rule>) -> MDFile {
    debug_assert!(pairs.as_rule() == Rule::md_file);
    let mut result: MDFile = MDFile {
        yaml: None,
        blocks: Vec::new(),
        path: Default::default(),
    };

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::yaml => {
                result.yaml = Some(parse_yaml(pair));
            }
            Rule::block => {
                result.blocks.push(parse_block(pair));
            }
            Rule::EOI => {}
            _ => panic!("unexpected rule: {:?}", pair.as_rule()),
        }
    }
    result
}

#[derive(Debug)]

struct YAML {
    yaml: serde_yaml::Value,
}

fn parse_yaml(pair: pest::iterators::Pair<Rule>) -> YAML {
    debug_assert!(pair.as_rule() == Rule::yaml);

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::yaml_inner => {
                return YAML {
                    yaml: serde_yaml::from_str(pair_inner.as_str()).unwrap(),
                };
            }
            _ => panic!("unexpected rule: {:?}", pair_inner.as_rule()),
        }
    }
    panic!("pairs inner is empty");
}

#[derive(Debug)]
enum Block {
    BlockQuote(BlockQuote),
    Latex(LatexBlock),
    Code(CodeBlock),
    String(StringBlock),
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Block {
    debug_assert!(pair.as_rule() == Rule::block || pair.as_rule() == Rule::yaml);

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::block_quote_block => {
                return Block::BlockQuote(parse_block_quote_block(pair_inner));
            }
            Rule::latex_block => {
                return Block::Latex(parse_latex_block(pair_inner));
            }
            Rule::code_block => {
                return Block::Code(parse_code_block(pair_inner));
            }
            Rule::string_block => {
                return Block::String(parse_string_block(pair_inner));
            }
            _ => panic!("unexpected rule: {:?}", pair_inner.as_rule()),
        }
    }
    panic!("pairs inner is empty");
}

fn parse_vec_line_into_block(pairs: Vec<pest::iterators::Pair<Rule>>) -> StringBlock {
    for pair in &pairs {
        debug_assert!(pair.as_rule() == Rule::block || pair.as_rule() == Rule::yaml);
    }
    let mut lines: Vec<Line> = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::line => {
                lines.push(parse_line(pair));
            }
            _ => panic!("unexpected rule: {:?}", pair.as_rule()),
        }
    }
    StringBlock { lines }
}

// not including >
#[derive(Debug)]
struct BlockQuote {
    inner_blocks: Vec<Block>,
}

fn parse_block_quote_block(pair: pest::iterators::Pair<Rule>) -> BlockQuote {
    debug_assert!(pair.as_rule() == Rule::block_quote_block);

    let mut inner_blocks: Vec<Block> = Vec::new();
    let lines: Vec<pest::iterators::Pair<Rule>> = pair.into_inner().collect();
    parse_block_quote_lines(lines);

    BlockQuote { inner_blocks }
}

#[derive(Debug)]

enum BlockQuoteLineState {
    Start,
    Line,
    BlockQuote,
}

fn parse_block_quote_lines(pairs: Vec<pest::iterators::Pair<Rule>>) -> BlockQuote {
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
                    _ => panic!("unexpected rule: {:?}", pair_inner),
                },
                BlockQuoteLineState::Line => match pair_inner.as_rule() {
                    Rule::block_quote_line => {
                        inner_blocks.push(Block::String(parse_vec_line(current_block)));
                        current_block = Vec::new();
                        current_block.push(pair_inner);
                        state = BlockQuoteLineState::BlockQuote;
                    }
                    Rule::line => {
                        current_block.push(pair_inner);
                    }
                    _ => panic!("unexpected rule: {:?}", pair_inner),
                },
                BlockQuoteLineState::BlockQuote => match pair_inner.as_rule() {
                    Rule::block_quote_line => {
                        current_block.push(pair_inner);
                    }
                    Rule::line => {
                        let block_quote = parse_block_quote_lines(current_block);
                        inner_blocks.push(Block::BlockQuote(block_quote));
                        current_block = Vec::new();
                        current_block.push(pair_inner);
                        state = BlockQuoteLineState::Line;
                    }
                    _ => panic!("unexpected rule: {:?}", pair_inner),
                },
            }
        }
    }
    match state {
        BlockQuoteLineState::Start => {}
        BlockQuoteLineState::Line => {
            if !current_block.is_empty() {
                inner_blocks.push(Block::String(parse_vec_line(current_block)));
            }
        }
        BlockQuoteLineState::BlockQuote => {
            if !current_block.is_empty() {
                let block_quote = parse_block_quote_lines(current_block);
                inner_blocks.push(Block::BlockQuote(block_quote));
            }
        }
    }

    BlockQuote { inner_blocks }
}

fn parse_vec_line(pairs: Vec<pest::iterators::Pair<Rule>>) -> StringBlock {
    for pair in &pairs {
        debug_assert!(pair.as_rule() == Rule::line);
    }
    let mut lines: Vec<Line> = Vec::new();
    for pair in pairs {
        lines.push(parse_line(pair));
    }
    StringBlock { lines }
}

// not including $$
#[derive(Debug)]
struct LatexBlock {
    latex: String,
}

fn parse_latex_block(pair: pest::iterators::Pair<Rule>) -> LatexBlock {
    debug_assert!(pair.as_rule() == Rule::latex_block);

    let latex = pair.as_str();

    LatexBlock {
        latex: latex.to_string(),
    }
}

// not including ```
#[derive(Debug)]
struct CodeBlock {
    code_type: Option<String>,
    code: String,
}

fn parse_code_block(pair: pest::iterators::Pair<Rule>) -> CodeBlock {
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
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    CodeBlock { code_type, code }
}
#[derive(Debug)]

struct StringBlock {
    lines: Vec<Line>,
}

fn parse_string_block(pair: pest::iterators::Pair<Rule>) -> StringBlock {
    debug_assert!(pair.as_rule() == Rule::string_block);

    let mut lines: Vec<Line> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::line => {
                lines.push(parse_line(pair_inner));
            }
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    StringBlock { lines }
}

#[derive(Debug)]
enum Line {
    NumberedList(NumberedList),
    BulletedList(BulletedList),
    Heading(Heading),
    StringLine(StringLine),
}

fn parse_line(pair: pest::iterators::Pair<Rule>) -> Line {
    debug_assert!(pair.as_rule() == Rule::line);

    let mut result: Line = Line::StringLine(StringLine { nodes: Vec::new() });

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::heading_line => {
                result = Line::Heading(parse_heading_line(pair_inner));
            }
            Rule::numbered_list_line => {
                result = Line::NumberedList(parse_numbered_list_line(pair_inner));
            }
            Rule::list_line => {
                result = Line::BulletedList(parse_list_line(pair_inner));
            }
            Rule::string_line => {
                result = Line::StringLine(parse_string_line(pair_inner));
            }
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    result
}

#[derive(Debug)]

struct NumberedList {
    indent: String,
    number: u32,
    nodes: Vec<Node>,
}

fn parse_numbered_list_line(pair: pest::iterators::Pair<Rule>) -> NumberedList {
    debug_assert!(pair.as_rule() == Rule::numbered_list_line);

    let mut indent: String = String::new();
    let mut number: u32 = 0;
    let mut nodes: Vec<Node> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::string_line => {
                nodes = parse_string_line(pair_inner).nodes;
            }
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    NumberedList {
        indent,
        number,
        nodes,
    }
}

#[derive(Debug)]

struct BulletedList {
    indent: String,
    nodes: Vec<Node>,
}

fn parse_list_line(pair: pest::iterators::Pair<Rule>) -> BulletedList {
    debug_assert!(pair.as_rule() == Rule::list_line);

    let mut indent: String = String::new();
    let mut nodes: Vec<Node> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::string_line => {
                nodes = parse_string_line(pair_inner).nodes;
            }
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    BulletedList { indent, nodes }
}

#[derive(Debug)]

struct Heading {
    level: u32,
    nodes: Vec<Node>,
}

fn parse_heading_line(pair: pest::iterators::Pair<Rule>) -> Heading {
    debug_assert!(pair.as_rule() == Rule::heading_line);

    let mut level: u32 = 0;
    let mut nodes: Vec<Node> = Vec::new();

    for pair_inner in pair.into_inner() {
        match pair_inner.as_rule() {
            Rule::string_line => {
                nodes = parse_string_line(pair_inner).nodes;
            }
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    Heading { level, nodes }
}

#[derive(Debug)]

struct StringLine {
    nodes: Vec<Node>,
}

fn parse_string_line(pair: pest::iterators::Pair<Rule>) -> StringLine {
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
                nodes.push(Node::NamedMDLink(parse_named_link_node(pair_inner)));
            }
            Rule::link_node => {
                nodes.push(Node::MDLink(pair_inner.as_str().to_string()));
            }
            Rule::weblink_node => {
                nodes.push(Node::WebLink(parse_weblink_node(pair_inner)));
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
            _ => panic!("unexpected rule: {:?}", pair_inner.as_rule()),
        }
    }

    StringLine { nodes }
}

#[derive(Debug)]

enum Node {
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

#[derive(Debug)]

struct NamedMDLink {
    name: String,
    link: String,
}

fn parse_named_link_node(pair: pest::iterators::Pair<Rule>) -> NamedMDLink {
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
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    NamedMDLink { name, link }
}

#[derive(Debug)]

struct WebLink {
    name: String,
    link: String,
}

fn parse_weblink_node(pair: pest::iterators::Pair<Rule>) -> WebLink {
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
            _ => panic!("unexpected rule: {:?}", pair_inner),
        }
    }

    WebLink { name, link }
}
