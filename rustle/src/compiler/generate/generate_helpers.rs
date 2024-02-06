use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::{EsVersion, Expr};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter, Node};

pub fn expr_to_string(expr: &Expr) -> String {
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
        Node::emit_with(&expr, &mut emmiter).unwrap();
    }

    let expr_literal = String::from_utf8(buffer).unwrap();
    expr_literal
}