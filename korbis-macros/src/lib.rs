use proc_macro::TokenStream;
use syn::{parse_macro_input, Error, LitInt};

mod offset;

#[proc_macro_attribute]
pub fn offset(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as LitInt);
    let item = parse_macro_input!(item as self::offset::OffsetItem);

    self::offset::transform(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
