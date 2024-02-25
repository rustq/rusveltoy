use crate::compiler::{Fragment, RustleAttribute, RustleElement};

use super::parser::Parser;
use super::swc_helpers::{parse_expression_at, swc_parse};

use lazy_static::lazy_static;
use regex::Regex;
use swc_common::Span;
use swc_ecma_ast::{Expr, Script};
use swc_html_ast::Text;

lazy_static! {
    static ref COMPONENT_TAG_PREFIX: Regex = Regex::new("[A-Z]").unwrap();
    static ref ELEMENT_TAG_NAME: Regex = Regex::new("[a-z]").unwrap();
    static ref ATTRIBUTE_NAME: Regex = Regex::new("[^=]").unwrap();
    static ref READ_TEXT: Regex = Regex::new("[^<{]").unwrap();
}

/// Parses fragments given an end condition.
///
/// # Arguments
/// * `parser` - The `parser` struct containing the content to parse
/// * `condition` - A closure that accepts a `Parser` argument and checks when to end the parsing
pub fn parse_fragments<F: Fn(&mut Parser) -> bool>(
    parser: &mut Parser,
    condition: F,
) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    while condition(parser) {
        if let Some(fragment) = parse_fragment(parser) {
            fragments.push(fragment);
        }
    }

    fragments
}

/// Parses a fragment given a parser struct. If it can't parse
/// it returns `None`.
///
/// # Arguments
/// * `parser` - The `parser` struct containing the content to parse.
pub fn parse_fragment(parser: &mut Parser) -> Option<Fragment> {
    if let Some(script) = parse_script(parser) {
        return Some(Fragment::Script(script));
    }

    if let Some(element) = parse_element(parser) {
        return Some(Fragment::Element(element));
    }

    if let Some(expression) = parse_expression(parser) {
        return Some(Fragment::Expression(expression));
    }

    if let Some(text) = parse_text(parser) {
        return Some(Fragment::Text(text));
    }

    None
}

/// Checks if the index starts at a `<script>` tag and parses
/// the content between it and a `</script>` tag using SWC
/// and returns a `swc_ecma_ast::Script`.
///
/// Sets the `parser.index` to the end of the closing `</script>` tag.
///
/// Returns `None` if the current index doesn't start at a `<script>` tag.
fn parse_script(parser: &mut Parser) -> Option<Script> {
    if parser.match_str("<script>") {
        parser.eat("<script>");
        let start_index = parser.index;
        let end_index = parser.content.find("</script>").unwrap();
        let code = parser.content.get(start_index..end_index).unwrap();
        let script = swc_parse(code);

        parser.index = end_index;
        parser.eat("</script>");

        return Some(script);
    }

    None
}

/// Checks if the index starts at an opening `<` tag
/// and parses the tag name and attributes.
///
/// A valid element tag looks like `<button on:click={action}>Button</button>`
/// or a simple `<div></div>.
///
/// Also parses recursively into elements like `<div><ul><li></li></ul></div>.
///
/// Sets the `parser.index` to the ending of the element.
fn parse_element(parser: &mut Parser) -> Option<RustleElement> {
    if parser.match_str("<") {
        parser.eat("<");

        if parser.match_str("!--") {
            parser.eat("!--");
            while !Regex::new(r"-->").unwrap().is_match(&parser.content[parser.index..parser.index+3]) {
                parser.index += 1;
            }
            parser.eat("-->");
            return None;
        }

        let prefix = parser.read_while_matching(&COMPONENT_TAG_PREFIX);
        let tag_name = parser.read_while_matching(&ELEMENT_TAG_NAME);
        let tag_name = format!("{}{}", prefix, tag_name);
        let attributes = parse_attribute_list(parser);
        let is_component = prefix.len() > 0;

        if parser.match_str("/>") {
            parser.eat("/>");

            let element = Some(RustleElement {
                name: tag_name,
                is_component,
                attributes: attributes,
                fragments: vec![],
            });
            return element;
        }

        parser.eat(">");

        let end_tag = format!("</{}>", tag_name);

        let element = Some(RustleElement {
            name: tag_name,
            is_component,
            attributes: attributes,
            fragments: parse_fragments(parser, |parser| !parser.match_str(&end_tag)),
        });

        parser.eat(end_tag.as_str());
        return element;
    }

    None
}

/// Checks if the index is at a curly brace `{` and parses the expression
/// at the index untill the next closing curly brace `}`.
///
/// Sets the `parser.index` to the closing curly brace `}` index.
fn parse_expression(parser: &mut Parser) -> Option<Expr> {
    if parser.match_str("{") {
        parser.eat("{");
        let expr = parse_javascript(parser);
        parser.eat("}");

        return Some(expr);
    }

    None
}

/// Parses text between tags for example `<div>some text</div>`
fn parse_text(parser: &mut Parser) -> Option<Text> {
    let text = parser.read_while_matching(&READ_TEXT);

    if text.trim() != "" {
        return Some(Text {
            span: Span::default(),
            data: text.into(),
            raw: None,
        });
    }
    None
}

/// Parses all the attributes inside a tag untill the closing `>`
/// for example `on:click={action}`
fn parse_attribute_list(parser: &mut Parser) -> Vec<RustleAttribute> {
    let mut attributes = Vec::new();
    parser.skip_whitespace();

    while !parser.match_str(">") && !parser.match_str("/>") {
        attributes.push(parse_attribute(parser));
        parser.skip_whitespace();
    }

    attributes
}

/// Gets the attribute name and the expression between curly braces
/// `on:click={action}` -> `on:click`, `action`
fn parse_attribute(parser: &mut Parser) -> RustleAttribute {

    // parse for attribute self case
    if parser.match_str("{") {
        parser.eat("{");
        let value = parse_javascript(parser);
        parser.eat("}");
        match value {
            Expr::Ident(ref ident) => {
                return RustleAttribute { name: ident.sym.to_string(), value }
            },
            _ => panic!()
        }
    }

    let name = parser.read_while_matching(&ATTRIBUTE_NAME);

    if parser.match_str("=\"") {
        parser.eat("=\"");
        parser.content.insert(parser.index, '`');
        let mut indent = 1usize;
        while parser.content.chars().nth(parser.index + indent).unwrap().to_string().as_str() != "\"" {
            indent += 1;
        }
        parser.content.insert(parser.index + indent, '`');

        let value = parse_javascript(parser);

        parser.eat("\"");

        return RustleAttribute { name, value };
    } else {
        parser.eat("={");

        let value = parse_javascript(parser);

        parser.eat("}");
        return RustleAttribute { name, value };
    }

}

/// Parses javascript using SWC at the current index.
/// Probably redundant.
fn parse_javascript(parser: &mut Parser) -> Expr {
    parse_expression_at(parser)
}
