#[cfg(test)]
mod tests {
    use rux_compiler::{Compiler, Lexer, Parser};

    #[test]
    fn test_lexer() {
        let source = r#"
            fn App() -> Element {
                <div>Hello</div>
            }
        "#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(!tokens.is_empty());
    }
    
    #[test]
    fn test_parser() {
        let source = r#"
            fn App() -> Element {
                <div>Hello</div>
            }
        "#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens, source.to_string());
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.items.len(), 1);
    }
    
    #[test]
    fn test_compiler() {
        let source = r#"
            fn App() -> Element {
                <div>Hello</div>
            }
        "#;
        
        let mut compiler = Compiler::new();
        let ast = compiler.compile_string(source, "test.rsx").unwrap();
        
        assert_eq!(ast.items.len(), 1);
    }
}
