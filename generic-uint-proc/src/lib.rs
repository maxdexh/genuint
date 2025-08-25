use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

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
        Ident::new(name, span).into(),
    ]
}
fn punct(ch: char, span: Span) -> TokenTree {
    spanned!(Punct::new(ch, Spacing::Alone), span)
}

fn compile_error(msg: &str, span: Span) -> TokenStream {
    pathseg("core", span)
        .into_iter()
        .chain(pathseg("compile_error", span))
        .chain([
            punct('!', span),
            spanned!(
                Group::new(
                    Delimiter::Brace,
                    [spanned!(Literal::string(msg), span)].into_iter().collect()
                ),
                span
            ),
        ])
        .collect()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn __apply(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut attr = attr.into_iter();
    let mut mac = Vec::new();
    for tok in &mut attr {
        if matches!(&tok, TokenTree::Punct(p) if p.as_char() == '!') {
            break;
        }
        mac.push(tok);
    }
    mac.push(Punct::new('!', proc_macro::Spacing::Alone).into());
    mac.push(
        Group::new(
            proc_macro::Delimiter::Brace,
            core::iter::once(Group::new(proc_macro::Delimiter::Parenthesis, attr.collect()).into())
                .chain(input)
                .collect(),
        )
        .into(),
    );
    TokenStream::from_iter(mac)
}

#[doc(hidden)]
#[proc_macro]
pub fn __lit(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();

    let Some(mut lit) = input.next() else {
        return compile_error("Unexpected end of input", Span::call_site());
    };

    while let TokenTree::Group(g) = lit {
        let Ok([single]) = <[_; 1]>::try_from(Vec::from_iter(g.stream())) else {
            return compile_error("Input must be a single literal token tree", g.span());
        };

        lit = single
    }
    let TokenTree::Literal(lit) = lit else {
        panic!("Expected literal, got {lit:?}");
    };

    let span = lit.span();
    let lit = lit.to_string();
    let lit = lit.as_str();

    let crate_path = TokenStream::from_iter(input);

    let doit = |digits, radix| -> Result<_, Box<dyn std::error::Error>> {
        let bits = {
            let num = ibig::UBig::from_str_radix(digits, radix)?;
            (0..num.bit_len()).map(move |i| num.bit(i))
        };
        let append_depth = bits.len();

        // [`crate::consts::_0`, `crate::consts::_1`]
        let mut consts = crate_path.clone();
        consts.extend(pathseg("consts", span));
        let consts = [(consts.clone(), false), (consts, true)].map(|(mut consts, bit)| {
            consts.extend(pathseg(if bit { "_1" } else { "_0" }, span));
            consts
        });

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
        let mut append = crate_path;
        append.extend(
            pathseg("ops", span)
                .into_iter()
                .chain(pathseg("AppendBit", span))
                .chain([punct('<', span)]),
        );

        // `crate::ops::AppendBit<..., crate::consts::_X>`
        Ok(core::iter::repeat_n(append, append_depth)
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
