use std::iter;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

trait ExtendExt<T>: Extend<T> + Sized {
    fn extended(mut self, iter: impl IntoIterator<Item = T>) -> Self {
        self.extend(iter);
        self
    }
}
impl<T, I: Extend<T>> ExtendExt<T> for I {}

macro_rules! spanned {
    ($ex:expr, $span:expr) => {{
        let mut tok = $ex;
        tok.set_span($span);
        ::proc_macro::TokenTree::from(tok)
    }};
}
fn pathseg(name: &str, span: Span) -> [TokenTree; 3] {
    [
        spanned!(Punct::new(':', Spacing::Joint), span),
        spanned!(Punct::new(':', Spacing::Alone), span),
        ident(name, span),
    ]
}
fn punct(ch: char, span: Span) -> TokenTree {
    spanned!(Punct::new(ch, Spacing::Alone), span)
}
fn group(stream: TokenStream, delim: Delimiter, delim_span: Span) -> TokenTree {
    spanned!(Group::new(delim, stream), delim_span)
}
fn litstr(str: &str, span: Span) -> TokenTree {
    spanned!(Literal::string(str), span)
}
fn ident(name: &str, span: Span) -> TokenTree {
    Ident::new(name, span).into()
}

fn compile_error(msg: &str, span: Span) -> TokenStream {
    pathseg("core", span)
        .into_iter()
        .chain(pathseg("compile_error", span))
        .chain([
            punct('!', span),
            spanned!(Group::new(Delimiter::Brace, litstr(msg, span).into()), span),
        ])
        .collect()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn __apply(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut attr = Vec::from_iter(attr);

    let args = attr
        .iter()
        .position(|tok| matches!(&tok, TokenTree::Punct(p) if p.as_char() == '!'))
        .map(|i| attr.split_off(i + 1))
        .unwrap_or_else(|| {
            attr.push(punct('!', Span::call_site()));
            Default::default()
        });

    attr.extended([group(
        TokenStream::from(group(
            TokenStream::from_iter(args),
            Delimiter::Parenthesis,
            Span::call_site(),
        ))
        .extended(input),
        Delimiter::Brace,
        Span::call_site(),
    )])
    .into_iter()
    .collect()
}

#[doc(hidden)]
#[proc_macro]
pub fn __lit(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();

    let Some(mut lit) = input.next() else {
        return compile_error("Unexpected end of input", Span::call_site());
    };

    while let TokenTree::Group(g) = lit {
        let mut tokens = g.stream().into_iter();
        let Some(single) = tokens.next() else {
            return compile_error("Unexpected end of input", g.span());
        };
        if let Some(leftover) = tokens.next() {
            return compile_error(
                "Leftover tokens. Input must be a single literal token without sign",
                leftover.span(),
            );
        }

        lit = single
    }
    let TokenTree::Literal(lit) = lit else {
        return compile_error("Expected literal", lit.span());
    };

    let span = lit.span();
    let lit = lit.to_string();
    let lit = lit.as_str();

    let crate_path = TokenStream::from_iter(input);

    let doit = |digits, radix| -> Result<_, Box<dyn std::error::Error>> {
        let bits = {
            let num = ibig::UBig::from_str_radix(digits, radix)?;
            (0..num.bit_len()).rev().map(move |i| num.bit(i))
        };
        let append_depth = bits.len();

        // [`crate::consts::_0`, `crate::consts::_1`]
        let consts = {
            let prefix = crate_path.clone().extended(pathseg("consts", span));
            [(prefix.clone(), "_0"), (prefix, "_1")]
                .map(|(c, name)| c.extended(pathseg(name, span)))
        };

        // first bit, `crate::consts::_0`
        let first = consts[0].clone();

        // [`, crate::consts::_0>`, `, crate::consts::_1`]
        let consts_and_puncts = consts.map(|c| {
            [punct(',', span)]
                .into_iter()
                .chain(c)
                .chain([punct('>', span)])
                .collect::<TokenStream>()
        });

        // `crate::ops::AppendBit<`
        let append = crate_path.extended(
            pathseg("ops", span)
                .into_iter()
                .chain(pathseg("AppendBit", span))
                .chain([punct('<', span)]),
        );

        // `crate::ops::AppendBit<..., crate::consts::_X>`
        Ok(iter::repeat_n(append, append_depth)
            .chain([first])
            .chain(bits.map(|bit| consts_and_puncts[usize::from(bit)].clone()))
            .collect())
    };

    match lit.split_at_checked(2) {
        Some(("0x", hex)) => doit(hex, 16),
        Some(("0o", oct)) => doit(oct, 8),
        Some(("0b", bin)) => doit(bin, 2),
        _ => doit(lit, 10),
    }
    .unwrap_or_else(|err| compile_error(&err.to_string(), span))
}
