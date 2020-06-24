mod lex;
#[macro_use]
mod tok;

use crate::ast::*;
pub use tok::Loc;
use tok::Token::*;
use tok::Misc::*;

type TokStream<'s> = std::iter::Peekable<lex::Lex<'s>>;



macro_rules! unexpected {
    ( $found: expr, $loc : expr, $expected: expr ) => 
    {return Err((format!("Unexpected {:?} encountered while looking for {}",
            $found, $expected),Some($loc)))};
}

macro_rules! fail {
    ( $reason : expr, $loc : expr ) => {return Err(($reason.to_string(),Some($loc)))};
    ( $reason : expr ) => {return Err(($reason.to_string(),None))};
}

type ParseError = (String,Option<tok::Loc>);

type ParseResult<T> = Result<T,ParseError>;

/* TODO: decide if this is worth salvaging
macro_rules! lookahead {
    ( $ts:ident, $loc:ident, 
      { $($cp:pat => $($cv:ident <- $cd:expr;)* $ce:expr,)* }, //consume
      { $($dp:pat => $($dv:ident <- $dd:expr;)* $de:expr,)* }, //descend
      $ifnone:expr) => { match $ts.peek() {
        Some(Ok((tok,$loc))) => match tok {
            $($cp => { $ts.next(); $(let $cv = $cd?;)* Ok($ce) },)*
            $($dp => { $(let $dv = $dd?;)* Ok($de) },)*
        },
        Some(Err((msg,loc))) => fail!(msg,loc),
        None => $ifnone,
    }
    }
}

macro_rules! parserule { // TODO: remove ambiguity
    ( $name:ident : $rettype:ty, $ts:ident, $loc:ident,
      { $($cp:pat => $($cv:ident <- $cd:expr;)* $ce:expr,)* }, //consume
      { $($dp:pat => $($dv:ident <- $dd:expr;)* $de:expr,)* }, //descend
      $ifnone:expr) => { 
        fn $name(ts: &mut TokStream) -> ParseResult<$rettype> {
            lookahead!($ts,$loc, 
                { $($cp => $($cv <- $cd;)* $ce,)*},
                { $($dp => $($dv <- $dd;)* $de,)*},
                $ifnone)
        }
    }
}
*/

macro_rules! disjunction_str {
    ($single:expr) => { $single };
    ($first:expr, $second:expr) => { concat!($first, " or ", $second) };
    ($lead:expr, $($multi:expr),+) => { concat!($lead, ", ", disjunction_str!($($multi),+)) };
}

macro_rules! tok_match {
    ( $tok:expr, 
      { $($p:pat $(, $subvar:pat = $subparse:expr)*$(,)? => $return:expr,)* }) => {
        match $tok {
            $($p => {$(let $subvar = $subparse?;)* $return },)*
            x => unexpected!(x,loc,disjunction_str!($(tokpat_to_str!($p)),*)),
        }
    };
}

macro_rules! maybe_tok_match {
    ( $maybetok:expr, $loc:ident,
      { $($p:pat $(, $subvar:pat = $subparse:expr)*$(,)? => $return:expr,)* },
      $ifnone:expr $(,)?) => {
        match $maybetok {
            Some(Ok((tok,$loc))) => tok_match!(tok, 
                { $($p $(, $subvar = $subparse)* => $return,)* }) ,
        Some(Err((msg,loc))) => fail!(msg,loc),
        None => $ifnone,
        }
    }
}

macro_rules! next_tok_match {
    ( $ts:ident, $loc:ident,
      { $($p:pat $(, $subvar:pat = $subparse:expr)*$(,)? => $return:expr,)* },
      $ifnone:expr $(,)?) => {
        maybe_tok_match!($ts.next(),$loc,
                { $($p $(, $subvar = $subparse)* => $return,)* },
                $ifnone)
    }
}

macro_rules! eatrule {
    ( $name:ident ($ts:ident $(,$arg:ident : $argtype:ty)*) -> $retty:ty { 
        match (tok at $loc:ident) 
            { $($p:pat $(, $subvar:pat = $subparse:expr)*$(,)? => $return:expr,)*
              | $ifnone:expr $(,)?}
    }) => { 
        fn $name($ts:&mut TokStream $(,$arg : $argtype)*) -> ParseResult<$retty> {
            next_tok_match!($ts,$loc, 
                { $($p $(, $subvar = $subparse)* => $return,)* }, 
                $ifnone,)
        }
    };
}

macro_rules! peekrule {
    ( $name:ident ($ts:ident $(,$arg:ident : $argtype:ty)*) -> $retty:ty { 
        match (tok at $loc:ident) 
            { $($p:pat $(, $subvar:pat = $subparse:expr)*$(,)? => $return:expr,)*
              | $ifnone:expr $(,)?}
    }) => { 
        fn $name($ts:&mut TokStream $(,$arg : $argtype)*) -> ParseResult<$retty> {
            maybe_tok_match!($ts.peek(),$loc, 
                { $($p $(, $subvar = $subparse)* => $return,)* }, 
                $ifnone,)
        }
    };
}

/* TODO: update and rewrite
pub fn spl_parse(source: &str) -> ParseResult<SPL> {
    let mut tokstream = lex::Lex::lex(source).peekable();
    let mut ast = Vec::new();
    let ts = &mut tokstream;
    while let Some(Ok((tok,loc))) = ts.peek(){
        ast.push(match tok {
            Marker(Var) => {ts.next(); let (v,e) = var_init(ts)?; Decl::Var(None, v, e)},
            TypeTok(_) | Marker(ParenOpen) |  Marker(BrackOpen) => {
                let typ = non_id_type(ts)?;
                let (v,e) = var_init(ts)?;
                Decl::Var(Some(typ),v,e)
            },
            IdTok(i) => fun_or_named_type_var_decl(ts)?,
            x => unexpected!(x,*loc,
                "'var', 'Int', 'Bool', 'Char', '(', '[', or identifier")
        }
        );

    }
    match tokstream.next() {
        None => Ok(ast),
        Some(Err((msg,loc))) => fail!(msg,loc),
        Some(Ok(_)) => panic!("`Some(Ok(_))` after `while let Some(Ok(_))`"),
    }
}
*/

/*
parserule!{decl : Option<Decl>, ts, loc, { 
    Marker(Var) => ve <- var_init(ts); Some(Decl::Var(None, ve.0, ve.1)),
    IdTok(i) => fun_or_named_type_var_decl(ts,i),
    },{ 
    TypeTok(_) | Marker(ParenOpen) | Marker(BrackOpen) => 
        typ <- non_id_type(ts);
        ve <- var_init(ts);
        Some(Decl::Var(Some(typ),ve.0,ve.1)),
    },
    Ok(None)
}
*/

fn var_init(ts: &mut TokStream) -> ParseResult<(Id,Exp)> {
    unimplemented!();
}

fn non_id_type(ts: &mut TokStream) -> ParseResult<Type> {
    unimplemented!();
}

fn fun_or_named_type_var_decl(ts: &mut TokStream) -> ParseResult<Decl> {
    match ts.next(){
        None => fail!("Ran out of tokens while parsing declaration with named type"),
        Some(Ok((tok,loc))) => unimplemented!(), // parse
        Some(Err(e)) => unimplemented!() //unexpected!(),
    }
}

eatrule!{ atom(ts) -> Exp {
    match(tok at loc) {
        IdTok(i) => field_or_call(ts,(i,Some(loc.into()))),
        Lit(val) => Ok(((BareExp::Lit(val), None), Some(loc.into()))),
        Marker(ParenOpen), 
            (coords, span) = tuplish(ts, exp),
            => ((BareExp::Tuple(coords), None), Some(hull(loc.into(),span))),
        | fail!("EOF while looking for primitive expression"),
    }
}
}


peekrule! { field_or_call(ts, id:Id) -> Exp {
    match(tok at loc) {
        Marker(ParenOpen), 
            _ = ts.next(),
            (args, end) = tuplish(ts, exp),
            => Ok(((Call(id,args), None), Some(hull(id.1,end)))),
        x, (fld, end) = field(ts,x) 
            => Ok(((BareExp::Var(id,fld),None), Some(hull(id.1,end)))),
        | Ok(((BareExp::(Var,Vec::new()),None), Some(id.1))),
    }
}
}

fn tuplish<T>(ts: &mut TokStream, single: fn(&mut TokStream) -> T) -> ParseResult<(Vec<T>,Span)> {
    todo!()
}


#[cfg(test)]
mod tests {
    use super::lex::*;
    #[test]
    fn test_sanity() {
        assert!(true);
    }
    
}
