use proc_macro::{Group, Punct, TokenStream, TokenTree};

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

// TODO: Literal parser
