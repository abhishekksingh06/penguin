use ginto_diag::Spanned;

#[derive(Clone, Debug)]
pub enum Type {
    Unit,
    Bool,
    U64,
    I64,
}

#[derive(Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEq,
    Less,
    Le,
    Greater,
    Ge,
    Or,
    And,
}

impl BinOp {
    pub fn binding_power(&self) -> (u8, u8) {
        match self {
            BinOp::Or => (1, 2),
            BinOp::And => (3, 4),
            BinOp::Equal | BinOp::NotEq => (5, 6),
            BinOp::Less | BinOp::Le | BinOp::Greater | BinOp::Ge => (7, 8),
            BinOp::Add | BinOp::Sub => (9, 10),
            BinOp::Mul | BinOp::Div | BinOp::Mod => (11, 12),
        }
    }
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Int(u64),
    Bool(bool),
    Unit,

    Unary {
        op: Spanned<UnaryOp>,
        expr: Box<Expr>,
    },

    Binary {
        op: Spanned<BinOp>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Let {
        name: Spanned<String>,
        ty: Option<Spanned<Type>>,
        value: Box<Expr>,
    },

    Assign {
        name: Spanned<String>,
        value: Box<Expr>,
    },

    Ident {
        exprs: Vec<Expr>,
        tail: Box<Expr>,
    },

    Var(String),
}

pub type Expr = Spanned<ExprKind>;

#[derive(Clone, Debug)]
pub enum Param {
    Named {
        name: Spanned<String>,
        ty: Option<Spanned<Type>>,
    },
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: Spanned<String>,
    pub params: Vec<Spanned<Param>>,
    pub ty: Option<Spanned<Type>>,
    pub body: Expr,
}
