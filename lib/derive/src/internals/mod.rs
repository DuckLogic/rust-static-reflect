use std::str::FromStr;

use self::func::FuncArgs;
use proc_macro2::TokenStream;
use syn::{spanned::Spanned, DeriveInput, Item};

pub mod fields;
pub mod func;
mod utils;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Repr {
    C,
    Transparent,
    Integer { signed: bool, bits: u32 },
}

pub fn determine_repr(input: &DeriveInput) -> Result<Option<Repr>, syn::Error> {
    let mut result = None;
    for attr in &input.attrs {
        if attr.meta.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if result.is_some() {
                    return Err(meta.error("Encountered multiple repr(...) attributes"));
                }
                let s = meta.path.require_ident()?.to_string();
                result = Some(match &*s {
                    "C" => Repr::C,
                    "transparent" => Repr::Transparent,
                    "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                        let signed = match s.chars().next() {
                            Some('i') => true,
                            Some('u') => false,
                            _ => unreachable!(),
                        };
                        let bits = u32::from_str(&s[1..]).unwrap();
                        Repr::Integer { signed, bits }
                    }
                    _ => return Err(syn::Error::new(meta.path.span(), "Unknown #[repr])")),
                });
                Ok(())
            })?;
        }
    }
    Ok(result)
}

pub fn derive_reflect_func(args: FuncArgs, input: &Item) -> Result<TokenStream, ::syn::Error> {
    let result = self::func::handle_item(input, args)?;

    self::utils::debug_proc_macro("reflect_func", &self::utils::item_name(input), &result);

    Ok(result)
}
