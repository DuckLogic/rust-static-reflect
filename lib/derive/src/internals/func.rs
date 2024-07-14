use syn::{Error, FnArg, ReturnType, ForeignItem, Attribute, ItemFn, Meta, ItemForeignMod, Item, Lit, Type, Expr};
use syn::parse::{self, Parse, ParseStream};
use proc_macro2::{Ident, TokenStream, Span};
use quote::{ToTokens, TokenStreamExt};
use syn::Signature;
use syn::spanned::Spanned;
use itertools::Itertools;
use quote::quote;


const FUNC_ATTR_NAME: &str = "reflect_func";

#[derive(Debug)]
#[non_exhaustive]
pub struct FuncArgs {
    /// Link against the hardcoded/absolute address
    /// instead of using dynamic linking
    pub absolute: bool,
}

impl Parse for FuncArgs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut args = FuncArgs {
            // By default, we want to use dynamic linking
            absolute: false,
        };
        while !input.is_empty() {
            if input.peek(syn::Ident) {
                let ident = input.parse::<Ident>()?;
                match &*ident.to_string() {
                    "absolute" => {
                        args.absolute = true;
                    }
                    _ => {
                        return Err(input.error(format_args!("Invalid flag: {}", ident)))
                    }
                }
            } else {
                return Err(input.error("Unexpected token"))
            }
        }
        Ok(args)
    }
}

#[derive(Debug, Clone)]
struct FunctionDefOpts {
    /// Assume that the function is already using the C ABI
    ///
    /// If this is "false" we require that the signature is explicitly declared `extern "C"`
    assume_c_abi: bool,
    /// The location of this function
    location: FunctionLocation,
    /// Whether the function is considered unsafe
    is_unsafe: bool,
}

/// Ensure that the function is either marked `#[no_mangle]`
/// or that it has a custom `#[export_name]`
fn determine_fn_link_name(item: &ItemFn) -> Result<Option<String>, Error> {
    for attr in &item.attrs {
        match attr.meta {
            Meta::Path(ref p) if p.is_ident("no_mangle") => {
                return Ok(None)
            },
            Meta::NameValue(ref item) if item.path.is_ident("export_name") => {
                return match item.value {
                    Expr::Lit(syn::ExprLit { lit: Lit::Str(ref s), .. }) => {
                        Ok(Some(s.value()))
                    },
                    _ => {
                        Err(Error::new(item.span(), "Expected a string for export_name"))
                    }
                }
            },
            _ => {}
        }
    }
    Err(Error::new(
        item.span(),
        "Function must be #[no_mangle] to support dynamic linking"
    ))
}

fn determine_foreign_link_name(attrs: &[Attribute]) -> Result<Option<String>, syn::Error> {
    for attr in attrs {
        match attr.meta {
            Meta::NameValue(ref l) if l.path.is_ident("link_name") => {
                return match l.value {
                    Expr::Lit(syn::ExprLit { lit: Lit::Str(ref s), .. }) => {
                        Ok(Some(s.value()))
                    },
                    _ => {
                        Err(syn::Error::new(
                            l.span(),
                            "Expected a string for #[link_name]"
                        ))
                    }
                }
            },
            _ => {}
        }
    }
    Ok(None)
}

pub fn handle_item(item: &Item, args: FuncArgs) -> Result<TokenStream, syn::Error> {
    match *item {
        Item::Fn(ref func) => handle_fn_def(func, args),
        Item::ForeignMod(ref foreign_mod) => handle_foreign_mod(foreign_mod, args),
        _ => {
            Err(Error::new(
                item.span(),
                format!("Invalid target for #[{}]", FUNC_ATTR_NAME)
            ))
        }
    }
}

fn handle_fn_def(item: &ItemFn, args: FuncArgs) -> Result<TokenStream, syn::Error> {
    let location = if args.absolute {
        let name = &item.sig.ident;
        FunctionLocation::AbsoluteAddress(quote!({ #name as *const () }))
    } else {
        let name = determine_fn_link_name(item)?;
        FunctionLocation::DynamicallyLinked {
            link_name: name.map(|s| quote!(#s))
        }
    };
    let def = emit_def_from_signature(&item.sig, FunctionDefOpts {
        assume_c_abi: false, location,
        is_unsafe: item.sig.unsafety.is_some()
    })?;
    let verify_types = types_from_signature(&item.sig);
    let def_const = def.make_constant(&verify_types);
    Ok(quote! {
        #def_const
        #item
    })
}

fn handle_foreign_mod(item: &ItemForeignMod, default_args: FuncArgs) -> Result<TokenStream, syn::Error> {
    // Handle default args
    if default_args.absolute {
        return Err(syn::Error::new(
            item.span(),
            "Absolute locations aren't supported in foreign functions"
        ));
    }
    match item.abi.name.as_ref() {
        Some(abi_name) if &*abi_name.value() == "C" => {},
        None => {},
        _ => {
            return Err(Error::new(item.abi.span(), "Expected C ABI"))
        }
    }
    let mut result_static_defs = Vec::new();
    let mut result_items = Vec::new();
    for item in &item.items {
        match *item {
            ForeignItem::Fn(ref item) => {
                let mut result_item = (*item).clone();
                result_item.attrs.clear();
                let mut override_args =None;
                for attr in &item.attrs {
                    if attr.path().is_ident(FUNC_ATTR_NAME) {
                        // NOTE: This attribute is removed from the result_item
                        attr.parse_nested_meta(|meta| {
                            if override_args.is_some() {
                                return Err(meta.error(format!("Conflicting #[{FUNC_ATTR_NAME}] attributes")));
                            }
                            override_args = Some(FuncArgs::parse(meta.input)?);
                            Ok(())
                        })?;
                    } else {
                        result_item.attrs.push(attr.clone());
                    }
                }
                // Handle overriding args
                if let Some(override_args) = override_args {
                    if override_args.absolute {
                        return Err(syn::Error::new(
                            item.span(),
                            "Absolute locations aren't supported in foreign functions"
                        ));
                    }
                }
                let link_name = determine_foreign_link_name(&item.attrs)?
                    .map(|s| quote!(#s));
                let args = FunctionDefOpts {
                    location: FunctionLocation::DynamicallyLinked { link_name },
                    assume_c_abi: true,
                    is_unsafe: true // All foreign defs are unsafe
                };
                let verify_types = types_from_signature(&item.sig);
                result_static_defs.push((
                    emit_def_from_signature(&item.sig, args)?,
                    verify_types
                ));
                result_items.push(ForeignItem::Fn(result_item));
            },
            _ => {
                // Passthrough
                result_items.push((*item).clone());
            }
        }
    }
    let function_def_consts = result_static_defs.iter()
        .map(|(def, verify_types)| def.make_constant(verify_types))
        .collect_vec();
    Ok(quote! {
        #(#function_def_consts)*
        extern "C" {
            #(#result_items)*
        }
    })
}

fn emit_def_from_signature(
    item: &Signature,
    opts: FunctionDefOpts,
) -> Result<StaticFunctionDef, syn::Error> {
    match item.abi.as_ref().and_then(|abi| abi.name.as_ref()) {
        Some(abi_name) if &*abi_name.value() == "C" => {},
        None if opts.assume_c_abi => {},
        _ => {
            return Err(Error::new(item.span(), "Expected C ABI"))
        }
    }
    let mut argument_types = Vec::new();
    let mut static_arg_types = Vec::new();
    for input in &item.inputs {
        match input {
            FnArg::Receiver(ref item) => {
                return Err(Error::new(item.span(), "Invalid input"))
            },
            FnArg::Typed(ref item) => {
                let ty = &item.ty;
                static_arg_types.push(quote!(#ty));
                argument_types.push(quote!(<#ty as static_reflect::StaticReflect>::TYPE_INFO))
            },
        }
    }
    let return_type = match item.output {
        ReturnType::Default => quote!(&static_reflect::types::TypeInfo::Unit),
        ReturnType::Type(_, ref ty) => {
            quote!(&<#ty as static_reflect::StaticReflect>::TYPE_INFO)
        },
    };
    let signature = StaticSignatureDef { argument_types, return_type };
    Ok(StaticFunctionDef {
        name: item.ident.to_string(),
        location: opts.location,
        signature, is_unsafe: opts.is_unsafe,
        static_return_type: match item.output {
            ReturnType::Default => quote!(()),
            ReturnType::Type(_, ref ty) => quote!(#ty),
        },
        static_arg_types: quote!((#(#static_arg_types,)*))
    })
}

// Get all the types from the signature
pub fn types_from_signature(sig: &Signature) -> Vec<Type> {
    sig.inputs.iter().map(|arg| match *arg {
        FnArg::Receiver(_) => Type::Verbatim(quote!(Self)),
        FnArg::Typed(ref t) => (*t.ty).clone()
    }).chain(std::iter::once(match sig.output {
        ReturnType::Default => Type::Tuple(syn::TypeTuple {
            paren_token: Default::default(),
            elems: Default::default()
        }),
        ReturnType::Type(_, ref ty) => (**ty).clone(),
    })).collect()
}

// Emit
#[derive(Clone, Debug)]
struct StaticFunctionDef {
    name: String,
    is_unsafe: bool,
    location: FunctionLocation,
    signature: StaticSignatureDef,
    static_return_type: TokenStream,
    static_arg_types: TokenStream,
}
impl StaticFunctionDef {
    fn make_constant(&self, verify_types: &[Type]) -> TokenStream {
        let const_name = format!("_FUNC_{}", self.name);
        let const_name = Ident::new(&const_name, Span::call_site());
        let def = self;
        let return_type = &self.static_return_type;
        let arg_types = &self.static_arg_types;
        quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            pub const #const_name: static_reflect::funcs::FunctionDeclaration<#return_type, #arg_types> = {
                // Verify all the types implement [StaticReflect]
                #(let _ = <#verify_types as static_reflect::StaticReflect>::TYPE_INFO;)*
                #def
            };
        }
    }
}

#[derive(Clone, Debug)]
struct StaticSignatureDef {
    argument_types: Vec<TokenStream>,
    return_type: TokenStream
}

#[derive(Clone, Debug)]
enum FunctionLocation {
    DynamicallyLinked {
        link_name: Option<TokenStream>
    },
    AbsoluteAddress(TokenStream),
}

impl ToTokens for StaticFunctionDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let StaticFunctionDef {
            ref name,
            ref signature,
            ref location,
            ref is_unsafe,
            ref static_return_type,
            static_arg_types: ref staitc_arg_types
        } = *self;
        tokens.append_all(quote!(static_reflect::funcs::FunctionDeclaration::<#static_return_type, #staitc_arg_types> {
            name: #name,
            is_unsafe: #is_unsafe,
            signature: #signature,
            location: #location,
            return_type: ::std::marker::PhantomData,
            arg_types: ::std::marker::PhantomData,
        }));
    }
}

impl ToTokens for FunctionLocation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match *self {
            FunctionLocation::DynamicallyLinked { link_name: None } => {
                quote!(Some(static_reflect::funcs::FunctionLocation::DynamicallyLinked { link_name: None }))
            },
            FunctionLocation::DynamicallyLinked { link_name: Some(ref name) } => {
                quote!(Some(static_reflect::funcs::FunctionLocation::DynamicallyLinked { link_name: Some(#name) }))
            },
            FunctionLocation::AbsoluteAddress(ref value) => {
                quote!(Some(static_reflect::funcs::FunctionLocation::AbsoluteAddress(#value)))
            },
        });
    }
}

impl ToTokens for StaticSignatureDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let StaticSignatureDef {
            ref argument_types,
            ref return_type
        } = *self;
        tokens.append_all(quote!(static_reflect::funcs::SignatureDef {
            argument_types: &[#(#argument_types),*],
            return_type: #return_type,
            // We use C FFI
            calling_convention: static_reflect::funcs::CallingConvention::StandardC
        }))
    }
}