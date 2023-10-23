use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Comma,
    Ident, LitInt, Index, Result,
};

struct AllTuplesWithIndex {
    macro_ident: Ident,
    start: usize,
    end: usize,
    idents: Vec<Ident>,
}

impl Parse for AllTuplesWithIndex {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let start = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let end = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let mut idents = vec![input.parse::<Ident>()?];
        while input.parse::<Comma>().is_ok() {
            idents.push(input.parse::<Ident>()?);
        }

        Ok(Self {
            macro_ident,
            start,
            end,
            idents,
        })
    }
}

/// Variant of bevy_utils::all_tuples for accessing the tuple members.
/// 
/// all_tuples_with_index!(impl_append, 1, 15, P, p);
/// // impl_append!((0, P0, p0));
/// // impl_append!((0, P0, p0), (1, P1, p1));
/// // impl_append!((0, P0, p0), (1, P1, p1), (2, P2, p2));
/// // ..
/// // impl_append!((0, P0, p0) .. (14, P14, p14));
/// ````
#[proc_macro]
pub fn all_tuples_with_index(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuplesWithIndex);
    let len = 1 + input.end - input.start;
    let mut ident_tuples = Vec::with_capacity(len);
    for i in 0..=len {
        let idents = input
            .idents
            .iter()
            .map(|ident| format_ident!("{}{}", ident, i));
        let idx = Index::from(i);
        ident_tuples.push(quote! {
            (#idx, #(#idents),*)
        });
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let ident_tuples = &ident_tuples[..i];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}
