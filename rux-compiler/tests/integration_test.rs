#[cfg(test)]
mod tests {
    use rux_compiler::{Compiler, Lexer, Parser, TypeChecker};

    #[test]
    fn test_simple_component() {
        let source = "fn App() -> Element { <div>Hello</div> }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens, source.to_string());
        let ast = parser.parse().unwrap();
        
        let mut type_checker = TypeChecker::new();
        type_checker.check(&ast).unwrap();
    }
}
