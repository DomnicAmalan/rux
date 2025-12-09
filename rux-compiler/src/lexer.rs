use crate::errors::{Error, Result};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Identifiers
    Ident(String),
    
    // Keywords
    Fn,
    If,
    Else,
    For,
    In,
    Let,
    Mut,
    Return,
    Match,
    Enum,
    Struct,
    Impl,
    Trait,
    Use,
    Pub,
    Mod,
    Const,
    Static,
    Type,
    Where,
    Async,
    Await,
    As,
    While,
    
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    Char(char),
    
    // Operators
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Eq,          // =
    EqEq,        // ==
    Ne,          // !=
    Lt,          // <
    Gt,          // >
    Le,          // <=
    Ge,          // >=
    And,         // &&
    Or,          // ||
    Not,         // !
    BitAnd,      // &
    BitOr,       // |
    BitXor,      // ^
    Shl,         // <<
    Shr,         // >>
    PlusEq,      // +=
    MinusEq,     // -=
    StarEq,      // *=
    SlashEq,     // /=
    PercentEq,   // %=
    Arrow,       // ->
    FatArrow,    // =>
    Dot,         // .
    DotDot,      // ..
    DotDotDot,   // ...
    Colon,       // :
    ColonColon,  // ::
    Semicolon,   // ;
    Comma,       // ,
    Question,    // ?
    
    // Punctuation
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]
    At,          // @
    Hash,        // #
    Dollar,      // $
    Underscore,  // _
    
    // JSX
    JSXOpen,      // <
    JSXClose,     // >
    JSXSlash,     // /
    JSXOpenTag(String),
    JSXCloseTag(String),
    JSXSelfClose, // />
    
    // Special
    Eof,
    Newline,
    Whitespace,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
    
    pub fn to_source_span(&self) -> miette::SourceSpan {
        (self.start, self.end - self.start).into()
    }
}

#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current: usize,
    line: usize,
    column: usize,
    start: usize,
    start_line: usize,
    start_column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            current: 0,
            line: 1,
            column: 1,
            start: 0,
            start_line: 1,
            start_column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<TokenWithSpan>> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let span = Span::new(
                self.start,
                self.current,
                self.start_line,
                self.start_column,
            );
            
            match &token {
                Token::Eof => {
                    tokens.push(TokenWithSpan { token, span });
                    break;
                }
                Token::Whitespace | Token::Newline => {
                    // Skip whitespace and newlines in token stream
                    continue;
                }
                _ => {
                    tokens.push(TokenWithSpan { token, span });
                }
            }
        }
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        self.start = self.current;
        self.start_line = self.line;
        self.start_column = self.column;
        
        let ch = match self.advance() {
            Some(ch) => ch,
            None => return Ok(Token::Eof),
        };
        
        match ch {
            // Whitespace and newlines (should be skipped, but handle if we get here)
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(Token::Newline)
            }
            ch if ch.is_whitespace() => Ok(Token::Whitespace),
            
            // Single character tokens
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            '[' => Ok(Token::LBracket),
            ']' => Ok(Token::RBracket),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            '@' => Ok(Token::At),
            '#' => Ok(Token::Hash),
            '$' => Ok(Token::Dollar),
            '_' => Ok(Token::Underscore),
            
            // Operators that might be multi-character
            '+' => {
                if self.match_char('=') {
                    Ok(Token::PlusEq)
                } else {
                    Ok(Token::Plus)
                }
            }
            '-' => {
                if self.match_char('=') {
                    Ok(Token::MinusEq)
                } else if self.match_char('>') {
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                if self.match_char('=') {
                    Ok(Token::StarEq)
                } else {
                    Ok(Token::Star)
                }
            }
            '/' => {
                if self.match_char('=') {
                    Ok(Token::SlashEq)
                } else if self.match_char('/') {
                    // Line comment
                    self.skip_line_comment();
                    self.next_token()
                } else if self.match_char('*') {
                    // Block comment
                    self.skip_block_comment()?;
                    self.next_token()
                } else {
                    Ok(Token::Slash)
                }
            }
            '%' => {
                if self.match_char('=') {
                    Ok(Token::PercentEq)
                } else {
                    Ok(Token::Percent)
                }
            }
            '=' => {
                if self.match_char('=') {
                    Ok(Token::EqEq)
                } else if self.match_char('>') {
                    Ok(Token::FatArrow)
                } else {
                    Ok(Token::Eq)
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(Token::Ne)
                } else {
                    Ok(Token::Not)
                }
            }
            '<' => {
                // Check if this is JSX
                if self.is_jsx_context() {
                    self.jsx_element()
                } else if self.match_char('=') {
                    Ok(Token::Le)
                } else if self.match_char('<') {
                    Ok(Token::Shl)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Token::Ge)
                } else if self.match_char('>') {
                    Ok(Token::Shr)
                } else {
                    Ok(Token::Gt)
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(Token::And)
                } else {
                    Ok(Token::BitAnd)
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(Token::Or)
                } else {
                    Ok(Token::BitOr)
                }
            }
            '^' => Ok(Token::BitXor),
            '.' => {
                if self.match_char('.') {
                    if self.match_char('.') {
                        Ok(Token::DotDotDot)
                    } else {
                        Ok(Token::DotDot)
                    }
                } else {
                    Ok(Token::Dot)
                }
            }
            ':' => {
                if self.match_char(':') {
                    Ok(Token::ColonColon)
                } else {
                    Ok(Token::Colon)
                }
            }
            '?' => Ok(Token::Question),
            
            // String literals
            '"' => self.string(),
            '\'' => self.char(),
            
            // Numbers
            ch @ '0'..='9' => {
                // Start number parsing with the digit we already consumed
                self.number_starting_with(ch)
            }
            
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                self.current -= 1;
                self.column -= 1;
                self.identifier_or_keyword()
            }
            
            _ => Err(Error::lexer(
                format!("Unexpected character: {}", ch),
                self.source,
                Span::new(self.start, self.current, self.start_line, self.start_column)
                    .to_source_span(),
            )),
        }
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.current += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }
    
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }
    
    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self) -> Result<()> {
        let mut depth = 1;
        while depth > 0 {
            match self.peek() {
                Some('*') => {
                    self.advance();
                    if self.match_char('/') {
                        depth -= 1;
                    }
                }
                Some('/') => {
                    self.advance();
                    if self.match_char('*') {
                        depth += 1;
                    }
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return Err(Error::lexer(
                        "Unterminated block comment",
                        self.source,
                        Span::new(self.start, self.current, self.start_line, self.start_column)
                            .to_source_span(),
                    ));
                }
            }
        }
        Ok(())
    }
    
    fn string(&mut self) -> Result<Token> {
        let mut value = String::new();
        
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(Token::String(value));
                }
                '\\' => {
                    self.advance(); // consume backslash
                    let escaped = self.advance().ok_or_else(|| Error::lexer(
                        "Unterminated string literal",
                        self.source,
                        Span::new(self.start, self.current, self.start_line, self.start_column)
                            .to_source_span(),
                    ))?;
                    value.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        '0' => '\0',
                        _ => escaped,
                    });
                }
                '\n' => {
                    return Err(Error::lexer(
                        "Unterminated string literal",
                        self.source,
                        Span::new(self.start, self.current, self.start_line, self.start_column)
                            .to_source_span(),
                    ));
                }
                _ => {
                    value.push(ch);
                    self.advance();
                }
            }
        }
        
        Err(Error::lexer(
            "Unterminated string literal",
            self.source,
            Span::new(self.start, self.current, self.start_line, self.start_column)
                .to_source_span(),
        ))
    }
    
    fn char(&mut self) -> Result<Token> {
        let ch = self.advance().ok_or_else(|| Error::lexer(
            "Unterminated character literal",
            self.source,
            Span::new(self.start, self.current, self.start_line, self.start_column)
                .to_source_span(),
        ))?;
        
        let value = if ch == '\\' {
            let escaped = self.advance().ok_or_else(|| Error::lexer(
                "Unterminated character literal",
                self.source,
                Span::new(self.start, self.current, self.start_line, self.start_column)
                    .to_source_span(),
            ))?;
            match escaped {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                '0' => '\0',
                _ => escaped,
            }
        } else {
            ch
        };
        
        if !self.match_char('\'') {
            return Err(Error::lexer(
                "Unterminated character literal",
                self.source,
                Span::new(self.start, self.current, self.start_line, self.start_column)
                    .to_source_span(),
            ));
        }
        
        Ok(Token::Char(value))
    }
    
    fn number_starting_with(&mut self, first_digit: char) -> Result<Token> {
        let mut value = String::new();
        value.push(first_digit);
        
        // Continue with rest of digits
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Fractional part
        if self.peek() == Some('.') {
            if let Some(next) = self.chars.clone().nth(1) {
                if next.is_ascii_digit() {
                    value.push('.');
                    self.advance();
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_digit() {
                            value.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        
        // Exponent
        if self.peek() == Some('e') || self.peek() == Some('E') {
            value.push(self.advance().unwrap());
            if self.peek() == Some('+') || self.peek() == Some('-') {
                value.push(self.advance().unwrap());
            }
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        let num = value.parse::<f64>().map_err(|_| Error::lexer(
            format!("Invalid number: {}", value),
            self.source,
            Span::new(self.start, self.current, self.start_line, self.start_column)
                .to_source_span(),
        ))?;
        
        Ok(Token::Number(num))
    }
    
    fn identifier_or_keyword(&mut self) -> Result<Token> {
        let mut ident = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check for keywords
        Ok(match ident.as_str() {
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "in" => Token::In,
            "let" => Token::Let,
            "mut" => Token::Mut,
            "return" => Token::Return,
            "match" => Token::Match,
            "enum" => Token::Enum,
            "struct" => Token::Struct,
            "impl" => Token::Impl,
            "trait" => Token::Trait,
            "use" => Token::Use,
            "pub" => Token::Pub,
            "mod" => Token::Mod,
            "const" => Token::Const,
            "static" => Token::Static,
            "type" => Token::Type,
            "where" => Token::Where,
            "async" => Token::Async,
            "await" => Token::Await,
            "as" => Token::As,
            "while" => Token::While,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            _ => Token::Ident(ident),
        })
    }
    
    fn is_jsx_context(&mut self) -> bool {
        // Simple heuristic: if next char is a letter or /, it's likely JSX
        matches!(self.peek(), Some('a'..='z' | 'A'..='Z' | '/'))
    }
    
    fn jsx_element(&mut self) -> Result<Token> {
        // This is a simplified JSX lexer
        // Full implementation would handle JSX attributes, children, etc.
        if self.match_char('/') {
            // Closing tag: </tag>
            let mut tag = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                    tag.push(ch);
                    self.advance();
                } else if ch == '>' {
                    self.advance();
                    return Ok(Token::JSXCloseTag(tag));
                } else {
                    break;
                }
            }
            return Err(Error::lexer(
                "Invalid JSX closing tag",
                self.source,
                Span::new(self.start, self.current, self.start_line, self.start_column)
                    .to_source_span(),
            ));
        } else {
            // Opening tag: <tag or <tag/>
            let mut tag = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                    tag.push(ch);
                    self.advance();
                } else if ch == '/' {
                    self.advance();
                    if self.match_char('>') {
                        return Ok(Token::JSXSelfClose);
                    }
                } else if ch == '>' {
                    self.advance();
                    return Ok(Token::JSXOpenTag(tag));
                } else {
                    break;
                }
            }
            Ok(Token::JSXOpen)
        }
    }
}
