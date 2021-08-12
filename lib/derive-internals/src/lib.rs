use std::str::FromStr;

use syn::{DeriveInput, Meta, NestedMeta, Item, spanned::Spanned};
use proc_macro2::{TokenStream};
use crate::func::FuncArgs;

pub mod func;
pub mod fields;
mod utils;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Repr {
    C,
    Transparent,
    Integer {
        signed: bool,
        bits: u32
    }
}

pub fn determine_repr(input: &DeriveInput) -> Result<Option<Repr>, syn::Error> {
    for attr in &input.attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if meta.path.is_ident("repr") && meta.nested.len() == 1 {
                if let NestedMeta::Meta(Meta::Path(path)) = &meta.nested[0] {
                    let ident =  path.get_ident().ok_or_else(|| syn::Error::new(path.span(), "Expected an identfier for #[repr(<name>))"))?;
                    let s = ident.to_string();
                    return Ok(match &*s {
                        "C" => Some(Repr::C),
                        "transparent" => Some(Repr::Transparent),
                        "u8" | "u16" | "u32" | "u64" |
                        "i8" | "i16" | "i32" | "i64" => {
                            let signed = match s.chars().next() {
                                Some('i') => true,
                                Some('u') => false,
                                _ => unreachable!()
                            };
                            let bits = u32::from_str(&s[1..]).unwrap();
                            Some(Repr::Integer { signed, bits })
                        },
                        _ => return Err(syn::Error::new(ident.span(), "Unknown #[repr])"))
                    })
                }
            }
        }
    }
    Ok(None)
}

pub fn derive_reflect_func(args: FuncArgs, input: &Item) -> Result<TokenStream, ::syn::Error> {
    let result = self::func::handle_item(&input, args)?;

    crate::utils::debug_proc_macro("reflect_func", &crate::utils::item_name(input), &result);

    Ok(result)
}

