use crate::lexer::Span;

#[derive(Debug, Clone)]
pub struct AST {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Component(Component),
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Impl(Impl),
    Use(Use),
    Mod(Mod),
    TypeAlias(TypeAlias),
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub props: Vec<Param>,
    pub return_type: Type,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        mutable: bool,
        span: Span,
    },
    Expr(Expr),
    Return(Option<Expr>, Span),
    If {
        condition: Expr,
        then: Box<Stmt>,
        else_: Option<Box<Stmt>>,
        span: Span,
    },
    For {
        var: String,
        iter: Expr,
        body: Box<Stmt>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
        span: Span,
    },
    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(String, Span),
    Literal(Literal, Span),
    Tuple(Vec<Pattern>, Span),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
        span: Span,
    },
    Wildcard(Span),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal, Span),
    Variable(String, Span),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
        span: Span,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    JSXElement(JSXElement, Span),
    Block(Block, Span),
    If {
        condition: Box<Expr>,
        then: Box<Expr>,
        else_: Option<Box<Expr>>,
        span: Span,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
        span: Span,
    },
    Tuple(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    Struct {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum JSXElement {
    SelfClosing {
        tag: String,
        props: Vec<JSXProp>,
        span: Span,
    },
    WithChildren {
        tag: String,
        props: Vec<JSXProp>,
        children: Vec<JSXChild>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum JSXChild {
    Element(JSXElement),
    Text(String, Span),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct JSXProp {
    pub name: String,
    pub value: JSXPropValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum JSXPropValue {
    Literal(Literal),
    Expr(Expr),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
    Neg,
    Deref,
    Ref,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Char(char),
    Unit,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Ident(String),
    Path(Vec<String>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Slice(Box<Type>),
    Reference {
        mutable: bool,
        inner: Box<Type>,
    },
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Option(Box<Type>),
    Result {
        ok: Box<Type>,
        err: Box<Type>,
    },
    Unit,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub data: Option<EnumVariantData>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum EnumVariantData {
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: String,
    pub items: Vec<TraitItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TraitItem {
    Method(Function),
    Type(String, Option<Type>),
}

#[derive(Debug, Clone)]
pub struct Impl {
    pub trait_name: Option<String>,
    pub type_name: String,
    pub items: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Use {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub aliased_type: Type,
    pub span: Span,
}
