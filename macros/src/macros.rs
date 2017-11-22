use proc_macro::{Literal, Span, TokenNode, TokenStream, TokenTree};
use syn::parse::{self, IResult};

pub fn ntstr(input: TokenStream) -> TokenStream {
    let (lit, span) = if let Some(lit) = literal(input) {
        lit
    } else {
        panic!("ntstr!() takes a single string literal");
    };
    let mut value = match parse::string(&lit.to_string()) {
        IResult::Done("", str_lit) => str_lit.value,
        IResult::Done(_, _) => unreachable!(),
        IResult::Error => panic!("ntstr!() takes a single string literal"),
    };
    if value.chars().any(|c| c == '\0') {
        panic!("ntstr!() literals should not contain any NUL byte");
    }
    value.push('\0');
    let kind = TokenNode::Literal(Literal::string(&value));
    TokenTree { span, kind }.into()
}

fn literal(input: TokenStream) -> Option<(Literal, Span)> {
    let group = match input.into_iter().next().unwrap().kind {
        TokenNode::Group(_, group) => group,
        _ => unreachable!(),
    };
    let mut iter = group.into_iter();
    if let (Some(token), None) = (iter.next(), iter.next()) {
        if let TokenNode::Literal(literal) = token.kind {
            Some((literal, token.span))
        } else {
            None
        }
    } else {
        None
    }
}
