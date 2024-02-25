use super::{analyse::AnalysisResult, Fragment, RustleAst};
use swc_common::{sync::Lrc, SourceMap};
use swc_ecma_ast::{EsVersion, Expr, Lit};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter, Node};

mod generate_helpers;
use generate_helpers::expr_to_string;

struct Code {
    counter: usize,
    variables: Vec<String>,
    components: Vec<(String, String)>,
    create: Vec<String>,
    update: Vec<String>,
    destroy: Vec<String>,
}

pub fn generate(ast: RustleAst, analysis: AnalysisResult) -> String {
    let mut code = Code {
        counter: 1,
        variables: Vec::new(),
        components: Vec::new(),
        create: Vec::new(),
        update: Vec::new(),
        destroy: Vec::new(),
    };

    for fragment in ast.fragments {
        traverse(&fragment, "target".into(), &analysis, &mut code)
    }

    let mut buffer = Vec::new();
    {
        let cm: Lrc<SourceMap> = Default::default();
        let writer = JsWriter::new(cm.clone(), "\n", &mut buffer, None);
        let config = Config {
            target: EsVersion::latest(),
            ascii_only: false,
            minify: false,
            omit_last_semi: false,
        };
        let mut emmiter = Emitter {
            cfg: config,
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        emmiter.emit_script(&ast.script).unwrap();
    }

    let script = String::from_utf8(buffer).unwrap();

    format!(
        r#"
	export default function() {{
		{}
		{}
		{}

		const lifecycle = {{
			create(target) {{
				{}
			}},
			update(changed) {{
				{}
			}},
			destroy() {{
				{}
			}},
		}};
		return lifecycle;
	}}
	"#,
        script,
        code.variables
            .iter()
            .map(|v| format!("let {};", v))
            .collect::<Vec<String>>()
            .join("\n"),
        code.components
            .iter()
            .map(|(v, component)| format!("let {} = new {}({{}});", v, component))
            .collect::<Vec<String>>()
            .join("\n"),
        code.create.join("\n"),
        code.update.join("\n"),
        code.destroy.join("\n")
    )
}

fn traverse(node: &Fragment, parent: String, analysis: &AnalysisResult, code: &mut Code) {
    match node {
        Fragment::Script(_) => (),
        Fragment::Element(f) => {
            let variable_name = format!("{}_{}", f.name, code.counter);
            code.counter += 1;

            match f.is_component {
                false => {
                    code.variables.push(variable_name.clone());
                    code.create.push(format!(
                        "{} = document.createElement('{}');",
                        variable_name, f.name
                    ));
                },
                true => {
                    code.components.push((variable_name.clone(), f.name.clone()));
                    code.create.push(format!(
                        "create_component({}.$$.fragment);",
                        variable_name.clone()
                    ));
                }
            }

            for attr in &f.attributes {
                if attr.name.starts_with("on:") {
                    let event_name = attr.name.chars().skip(3).collect::<String>();
                    let event_handler = match &attr.value {
                        Expr::Ident(ident) => ident.sym.to_string(),
                        _ => panic!(),
                    };

                    code.create.push(format!(
                        "{}.addEventListener('{}', {});",
                        parent, event_name, event_handler
                    ));

                    code.destroy.push(format!(
                        "{}.removeEventListener('{}', {});",
                        parent, event_name, event_handler
                    ));
                } else {
                    let mut value = match &attr.value {
                        Expr::Ident(ident) => ident.sym.to_string(),
                        Expr::Lit(Lit::Str(str)) => format!("\"{}\"", str.value),
                        expr => expr_to_string(expr),
                        _ => {
                            todo!()
                        },
                    };
                    {
                        // replace `{xx} xx` => `${xx} xx`
                        let mut index = 0usize;
                        while index < value.len() {
                            if value.chars().nth(index).unwrap() == '{' {
                                let mut next_pointer = index + 1;
                                while next_pointer < value.len() {
                                    if value.chars().nth(next_pointer).unwrap() == '}' {
                                        // match
                                        value.insert(index, '$');
                                        next_pointer += 1;
                                        index += 1;
                                    }
                                    next_pointer += 1;
                                }
                            }
                            index += 1;
                        }
                    }
                    code.create.push(format!(
                        "{}.{} = {};",
                        parent, attr.name, value
                    ));
                }
            }

            for fragment in &f.fragments {
                traverse(fragment, variable_name.clone(), analysis, code);
            }

            match f.is_component {
                false => {
                    code.create
                    .push(format!("{}.appendChild({});", parent, variable_name));
                    code.destroy
                    .push(format!("{}.removeChild({});", parent, variable_name));
                },
                true => {
                    code.create
                    .push(format!("mount_component({});", variable_name));
                    code.destroy
                    .push(format!("destroy_component({});", variable_name));

                }
            }
        }
        Fragment::Expression(f) => {
            let variable_name = format!("txt_{}", code.counter);
            code.counter += 1;

            let expression_name = match f {
                Expr::Ident(ident) => ident.sym.to_string(),
                expr => expr_to_string(expr),
                _ => panic!(),
            };

            code.variables.push(variable_name.clone());
            code.create.push(format!(
                "{} = document.createTextNode({});",
                variable_name, expression_name
            ));

            code.create
                .push(format!("{}.appendChild({});", parent, variable_name));

            if analysis.will_change.contains(&expression_name) {
                code.update.push(format!(
                    r#"
					if (changed.includes('{}')) {{
						{}.data = {};
					}}
				"#,
                    expression_name, variable_name, expression_name
                ));
            }
        }
        Fragment::Text(f) => {
            let variable_name = format!("txt_{}", code.counter);
            code.counter += 1;

            code.variables.push(variable_name.clone());
            code.create.push(format!(
                "{} = document.createTextNode(`{}`);",
                variable_name.clone(),
                f.data.to_string()
            ));
            code.create
                .push(format!("{}.appendChild({});", parent, variable_name));
        }
    }
}
