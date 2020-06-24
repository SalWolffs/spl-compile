use crate::ast::{BareId,BareSelector,BType,LitVal,BareOp};

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Loc{
    pub line: u32,
    pub col: u16,
    pub len: u16,
}

impl Loc { // TODO: guard against overflow
    pub fn next_line(&mut self) {
        self.line += 1;
        self.col = 0;
        self.len = 0;
    }

    pub fn advance(&mut self) {
        self.col += self.len;
        self.len = 0;
    }

    pub fn step(&mut self) {
        self.len += 1;
    }
}

pub type Located<T> = (T,Loc);

#[derive(Copy,Clone,Debug,PartialEq)]
pub(super) enum Misc {
    Var,
    If,
    Else,
    While,
    Return,
    Semicolon,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    BrackOpen,
    BrackClose,
    Comma,
    Dot,
    Arrow,
    Assign,
    TypeColon,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub(super) enum Token {
    IdTok(BareId),
    Selector(BareSelector),
    TypeTok(BType),
    Lit(LitVal),
    Op(BareOp),
    Marker(Misc),
}

pub(super) type LocTok = Located<Token>;

pub(super) trait TokAble  {
    fn to_tok(self) -> Token;
    fn to_ltok(self,l:Loc) -> LocTok 
        where Self: std::marker::Sized {
        (self.to_tok(),l)
    }
}

impl TokAble for u32 {
    fn to_tok(self) -> Token {
        Token::IdTok(self as BareId)
    }
}

impl TokAble for BareSelector {
    fn to_tok(self) -> Token {
        Token::Selector(self)
    }
}

impl TokAble for BType {
    fn to_tok(self) -> Token {
        Token::TypeTok(self)
    }
}

impl TokAble for LitVal {
    fn to_tok(self) -> Token {
        Token::Lit(self)
    }
}

impl TokAble for BareOp {
    fn to_tok(self) -> Token {
        Token::Op(self)
    }
}

impl TokAble for Misc {
    fn to_tok(self) -> Token {
        Token::Marker(self)
    }
}

#[macro_export]
macro_rules! tokpat_to_str {
    (Marker(Var)) => { "'var'" };
    (TypeTok(IntT)) => { "'Int'" };
    (TypeTok(BoolT)) => { "'Bool'" };
    (TypeTok(CharT)) => { "'Char'" };
    (TypeTok(UnitT)) => { "'Void'" };
    (TypeTok($t:pat)) => { "base type" };
    (Marker(BracOpen)) => { "'{'" };
    (Marker(ParenOpen)) => { "'('" };
    (IdTok($i:pat)) => { "identifier" };
    (Lit($v:pat)) => { "literal" };

}

mod test {
    use super::*;
    use super::Token::*;
    #[test]
    fn typetokpat_to_str() {
        assert_eq!(tokpat_to_str!(TypeTok(_)), "base type");
        assert_eq!(tokpat_to_str!(TypeTok(CharT)), "'Char'");
    }
    #[test]
    fn parenopen_to_str() {
        assert_eq!(tokpat_to_str!(Marker(ParenOpen)), "'('");
    }

    #[test]
    fn idtok_to_str() {
        assert_eq!(tokpat_to_str!(IdTok(i)), "identifier");
    }

    #[test]
    fn lit_to_str() {
        assert_eq!(tokpat_to_str!(Lit(val)), "literal");
    }
}
