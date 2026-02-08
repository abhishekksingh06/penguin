use std::process::exit;

use ginto_diag::{
    DiagnosticConvertible, DiagnosticRenderer, PlainDiagnosticRenderer, SourceFile, SourceManager,
};
use ginto_syntax::{Lexer, Parser};

fn main() {
    let mut sm = SourceManager::new();
    let input = "23+1@".to_string();
    let file = sm.add_file("main.ginto".to_string(), input.to_string());
    let mut lexer = Lexer::new(file, &input);
    let error_renderer = PlainDiagnosticRenderer;
    let tokens = match lexer.lex_all() {
        Ok(t) => t,
        Err(errs) => {
            for err in errs {
                let r = error_renderer.render(&sm, err.into_diagnostic());
                println!("{r}")
            }
            exit(1)
        }
    };
    let mut p = Parser::new(file, tokens);
    let errs = p.errors();
    if errs.is_empty() {
        let expr = p.parse_expr().unwrap();
        println!("{:#?}", expr);
    } else {
        for err in errs {
            let r = error_renderer.render(&sm, err.clone().into_diagnostic());
            println!("{r}");
        }
    }
}
