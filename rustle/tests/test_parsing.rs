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


// in browser passed
#[test]
fn test_parsing_demo() { test_parsing("demo".to_owned()) }


// in browser passed
#[test]
fn test_close_tag() { test_parsing("close-tag".to_owned()) }


// in browser passed
#[test]
fn test_attribute() { test_parsing("attribute".to_owned()) }


// in browser passed
#[test]
fn test_attribute_expr() { test_parsing("attribute_expr".to_owned()) }


// in browser passed
#[test]
fn test_attribute_self() { test_parsing("attribute_self".to_owned()) }


// in browser passed
#[test]
fn test_attribute_template_string() { test_parsing("attribute_template_string".to_owned()) }


// in browser passed
#[test]
fn test_parsing_hello() { test_parsing("hello".to_owned()) }

// in browser passed
#[test]
fn test_parsing_reactive_assignments() { test_parsing("reactive-assignments".to_owned()) }

// in browser passed
#[test]
fn test_parsing_nested() {
    test_parsing("nested".to_owned());
    let source = fs::read_to_string("tests/nested/Nested.rustle").unwrap();
    let ast = Parser::new(&source).parse();
    let analysis = analyse(&ast);
    let generated = generate(ast, analysis);

    fs::write("tests/nested/Nested.js", generated).unwrap();
}

// in browser passed
#[test]
fn test_tag_update() { test_parsing("tag_update".to_owned()) }

// in browser passed
#[test]
fn test_component_update() {
    test_parsing("component_update".to_owned());

    let source = fs::read_to_string("tests/component_update/Nested.rustle").unwrap();
    let ast = Parser::new(&source).parse();
    let analysis = analyse(&ast);
    let generated = generate(ast, analysis);

    fs::write("tests/component_update/Nested.js", generated).unwrap();
}
