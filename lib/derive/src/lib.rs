extern crate proc_macro;

mod internals;

use syn::{parse_macro_input, DeriveInput, Item};

#[proc_macro_derive(StaticReflect, attributes(reflect, static_reflect))]
pub fn derive_static_reflect(raw_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(raw_input as DeriveInput);
    match internals::fields::derive_static_reflect(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn reflect_func(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _original_input = input.clone();
    let input: Item = parse_macro_input!(input as Item);
    let args = parse_macro_input!(args as internals::func::FuncArgs);
    match internals::derive_reflect_func(args, &input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
