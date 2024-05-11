use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, Error, Ident, LitInt, Pat, PatType, Receiver, ReturnType, Token};

pub fn transform(args: LitInt, item: OffsetItem) -> syn::Result<TokenStream> {
    match item {
        OffsetItem::Method(v) => transform_method(args, v),
    }
}

fn transform_method(args: LitInt, item: Method) -> syn::Result<TokenStream> {
    // Assemble.
    let offset: usize = args.base10_parse()?;
    let unsafety = item.unsafety;
    let ident = item.ident;
    let receiver = item.receiver;
    let params = item.params;
    let ret = item.ret;
    let args: Punctuated<&Pat, Token![,]> = params.iter().map(|p| p.pat.as_ref()).collect();

    Ok(quote! {
        #unsafety fn #ident(#receiver, #params) #ret {
            let _addr = unsafe { self.elf().as_ptr().add(#offset) };
            let _fp: unsafe extern "C" fn(#params) #ret = unsafe { core::mem::transmute(_addr) };
            unsafe { _fp(#args) }
        }
    })
}

/// Item of `offset` attribute.
pub enum OffsetItem {
    Method(Method),
}

impl Parse for OffsetItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let unsafety = input.parse()?;
        let item = if input.parse::<Option<Token![fn]>>()?.is_some() {
            // Parse name.
            let ident = input.parse()?;
            let params;

            parenthesized!(params in input);

            // Parse receiver.
            let receiver = params.parse()?;

            params.parse::<Option<Token![,]>>()?;

            // Parse return type.
            let ret = input.parse()?;

            input.parse::<Token![;]>()?;

            Self::Method(Method {
                unsafety,
                ident,
                receiver,
                params: params.parse_terminated(PatType::parse, Token![,])?,
                ret,
            })
        } else {
            return Err(Error::new(input.span(), "unsupported offset item"));
        };

        Ok(item)
    }
}

/// A method that have `offset` attribute.
pub struct Method {
    unsafety: Option<Token![unsafe]>,
    ident: Ident,
    receiver: Receiver,
    params: Punctuated<PatType, Token![,]>,
    ret: ReturnType,
}
