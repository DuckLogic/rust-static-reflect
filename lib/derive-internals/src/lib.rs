use syn::{DeriveInput, Meta, NestedMeta, Item};
use proc_macro2::{TokenStream};
use crate::func::FuncArgs;

pub mod func;
pub mod fields;
mod utils;

pub fn has_repr_c(input: &DeriveInput) -> bool {
    for attr in &input.attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if meta.path.is_ident("repr") && meta.nested.len() == 1 {
                if let NestedMeta::Meta(Meta::Path(path)) = &meta.nested[0] {
                    if path.is_ident("C") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn derive_reflect_func(args: FuncArgs, input: &Item) -> Result<TokenStream, ::syn::Error> {
    let result = self::func::handle_item(&input, args)?;

    crate::utils::debug_proc_macro("reflect_func", &crate::utils::item_name(input), &result);

    Ok(result)
}

