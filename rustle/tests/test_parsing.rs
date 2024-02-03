use std::fs;

use rustle::compiler::analyse::analyse;
use rustle::compiler::generate::generate;
use rustle::compiler::parse::Parser;

fn test_parsing(path: String) {
    let source = fs::read_to_string(format!("tests/{}/app.rustle", path)).unwrap();
    let ast = Parser::new(&source).parse();
    let analysis = analyse(&ast);
    let generated = generate(ast, analysis);

    fs::write(format!("tests/{}/app.js", path), generated).unwrap();

    assert!(true)
}


#[test]
fn test_parsing_demo() { test_parsing("demo".to_owned()) }


#[test]
fn test_close_tag() { test_parsing("close-tag".to_owned()) }


#[test]
fn test_attribute() { test_parsing("attribute".to_owned()) }


#[test]
fn test_attribute_expr() { test_parsing("attribute_expr".to_owned()) }


#[test]
fn test_attribute_self() { test_parsing("attribute_self".to_owned()) }


/// to fix
#[test]
fn test_attribute_template_string() { test_parsing("attribute_template_string".to_owned()) }


// #[test]
// fn test_parsing_hello() { test_parsing("hello".to_owned()) }


// #[test]
// fn test_parsing_reactive_assignments() { test_parsing("reactive-assignments".to_owned()) }
