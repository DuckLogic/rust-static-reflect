use quote::{quote, format_ident};
use syn::{parenthesized, Token, parse_quote, DeriveInput, Data, Generics, GenericParam, TypeParamBound, DataEnum, DataStruct, DataUnion, Type};
use proc_macro2::{TokenStream, Ident, Span};
use syn::parse::{self, Parse, ParseStream};
use syn::spanned::Spanned;
use indexmap::IndexMap;

use super::{determine_repr, Repr};

#[derive(Debug)]
#[non_exhaustive]
pub struct DeriveFieldOptions {
    /// Treat this array field as opaque and unsized.
    ///
    /// For example,
    /// ````ignore
    /// struct PyTuple {
    ///     ob_refcnt: usize,
    ///     ob_size: usize,
    ///     #[reflect(opaque_array)]
    ///     ob_items: [PyObject; 1]
    /// }
    /// ````
    /// The final field will have type `PyObject` instead of `[PyObject; 1]`.
    /// The actual size won't really be known at compile time,
    /// so it's up to the user to ensure no attempts are made at stack allocation
    ///
    /// The array must be in trailing position
    pub opaque_array: bool,
    /// Assume the field has the same underlying representation as the specified type.
    ///
    /// Useful if the type is known to be FFI-safe,
    /// but the field's type doesn't actually implement `StaticReflect`
    pub assume_repr: Option<syn::Type>
}
impl DeriveFieldOptions {
    pub fn parse_attrs(attrs: &[syn::Attribute]) -> Result<DeriveFieldOptions, syn::Error> {
        for attr in attrs {
            if attr.path.is_ident("reflect") || attr.path.is_ident("static_reflect") {
                return syn::parse2(attr.tokens.clone())
            }
        }
        Ok(DeriveFieldOptions::default())
    }
}
impl Default for DeriveFieldOptions {
    fn default() -> DeriveFieldOptions {
        DeriveFieldOptions {
            // Most fields are not trailing arrays
            opaque_array: false,
            // This is unsafe
            assume_repr: None
        }
    }
}

impl Parse for DeriveFieldOptions {
    fn parse(bracketed_input: ParseStream) -> parse::Result<Self> {
        let mut args = DeriveFieldOptions::default();
        let input;
        parenthesized!(input in bracketed_input);
        let start_span = input.span();
        while !input.is_empty() {
            if input.peek(syn::Ident) {
                let ident = input.parse::<Ident>()?;
                match &*ident.to_string() {
                    "opaque_array" => {
                        args.opaque_array = true;
                    },
                    "assume_repr" => {
                        input.parse::<Token![=]>()?;
                        let type_str = input.parse::<syn::LitStr>()?;
                        let desired_type = syn::parse_str::<Type>(&type_str.value())
                            .map_err(|cause| syn::Error::new(
                                type_str.span(),
                                format_args!("Invalid type: {}", cause)
                            ))?;
                        args.assume_repr = Some(desired_type);
                    }
                    _ => {
                        return Err(input.error(format_args!("Invalid flag: {}", ident)))
                    }
                }
            } else {
                return Err(input.error("Unexpected token"))
            }
        }
        if args.assume_repr.is_some() && args.opaque_array {
            return Err(syn::Error::new(
                start_span, "opaque_array is incompatible with assume_repr",
            ))
        }
        Ok(args)
    }
}

pub fn derive_static_reflect(input: &DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;
    let repr = determine_repr(&input)?;
    if repr != Some(Repr::C) && !matches!(input.data, Data::Enum(_)) {
        return Err(syn::Error::new(
            name.span(),
            "StaticReflect requires repr(C)"
        ))
    }

    let generics = add_type_bounds(&input.generics, &[parse_quote!(::reflect::StaticReflect)]);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut extra_defs = Vec::new();
    let static_type = match input.data {
        Data::Struct(ref data) => {
            handle_type(
                StructHandler::new(data, name),
                &name,
                quote!(#impl_generics),
                quote!(#ty_generics),
                quote!(#where_clause),
                &mut extra_defs
            )?
        },
        Data::Enum(ref data) => enum_static_type(data, repr, &name)?,
        Data::Union(ref data) => {
            handle_type(
                UnionTypeHandler { data, name },
                &name,
                quote!(#impl_generics),
                quote!(#ty_generics),
                quote!(#where_clause),
                &mut extra_defs
            )?
        },
    };

    let r = quote! {
        #(#extra_defs)*
        unsafe impl #impl_generics static_reflect::StaticReflect for #name #ty_generics #where_clause {
            const TYPE_INFO: static_reflect::types::TypeInfo<'static> = {
                /*
                 * NOTE: All our fields are assumed to implement `StaticReflect`,
                 * because there is no other way they could show up
                 * in the generated `TypeInfo`.
                 */
                #static_type
            };
        }
    };
    super::utils::debug_derive("StaticReflect", &input.ident, &r);
    Ok(r)
}
fn handle_type<'a, T: TypeHandler<'a>>(
    mut target: T,
    name: &Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
    where_clause: TokenStream,
    extra_defs: &mut Vec<TokenStream>
) ->  Result<TokenStream, syn::Error> {
    let mut field_info: IndexMap<FieldName<'a>, TokenStream> = IndexMap::new();
    let mut field_associated_types = Vec::new();
    let mut field_defs = Vec::new();
    target.handle_fields(|field| {
        let field_name = field.name;
        let field_type = &field.static_type;
        field_info.insert(field_name.clone(), field.static_def.clone());
        let associated_type_name = field_name.associated_type_name();
        field_associated_types.push(quote!(type #associated_type_name = #field_type;));
        let field_def_type = T::field_def_type(Some(quote!(#field_type)));
        match field_name {
            FieldName::Tuple { index: _ } => {
                field_defs.push(quote!(pub #field_def_type));
            }
            FieldName::Named { name } => {
                field_defs.push(quote!(pub #name: #field_def_type));
            }
        }
    })?;
    let field_info_struct_name = Ident::new(
        &format!("_FieldInfo{}", name),
        name.span()
    );
    let field_info_trait_name = Ident::new(
        &format!("_FieldTrait{}", name),
        name.span()
    );
    let associated_type_names = field_info.keys().map(FieldName::associated_type_name);
    let field_info_struct_def = {
        let fields = quote!(#(#field_defs),*);
        let fields = if target.is_tuple_style() {
            // NOTE: I guess a tuple-struct needs a semicolon but a regular struct doesn't....
            quote!((#fields);)
        } else {
            quote!({ #fields })
        };
        quote! {
            #[allow(missing_docs)]
            #[doc(hidden)]
            pub struct #field_info_struct_name #fields
        }
    };
    extra_defs.push(quote!(
        #field_info_struct_def
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        trait #field_info_trait_name {
            #(type #associated_type_names;)*
        }
        #[allow(non_camel_case_types)]
        impl #impl_generics #field_info_trait_name for #name #ty_generics #where_clause {
            #(#field_associated_types)*
        }
    ));
    let field_inits = field_info.iter()
        .map(|(name, def)| match name {
            FieldName::Tuple { .. } => quote!(#def),
            FieldName::Named { name } => quote!(#name: #def)
        })
        .collect::<Vec<TokenStream>>();
    let field_inits = if target.is_tuple_style() {
        quote!((#(#field_inits,)*))
    } else {
        quote!({#(#field_inits,)*})
    };
    extra_defs.push(quote!(
        unsafe impl #impl_generics static_reflect::FieldReflect for #name #ty_generics #where_clause {
            type NamedFieldInfo = #field_info_struct_name;
            const NAMED_FIELD_INFO: Self::NamedFieldInfo = #field_info_struct_name #field_inits;
        }
    ));
    let field_access = field_info.keys().map(|name| name.access()).collect::<Vec<_>>();
    let field_def_type_name = T::field_def_type(None);
    let type_def_type = T::type_def_type();
    let header = quote! {
        use static_reflect::{StaticReflect, FieldReflect};
        use static_reflect::types::TypeInfo;
        use #field_def_type_name;
        use #type_def_type;
        const _FIELDS: &'static [#field_def_type_name<'static>] = &[#(<#name as FieldReflect>::NAMED_FIELD_INFO.#field_access.erase()),*];
    };
    let static_def = target.create_static_def(header);
    let into_type = T::def_into_type(quote!(_DEF));
    Ok(quote!({
        const _DEF: &'static #type_def_type<'static> = &#static_def;
        #into_type
    }))
}
fn is_c_style_enum(data: &DataEnum) -> bool {
    /*
     * TODO: Should the following be considered a 'c-style' enum?
     * enum Test {
     *     One,
     *     Two {  },
     *     Three( )
     * }
     *
     * Right now it *is*, because `fields.is_empty`,
     * although we could require them all to be unit-variants (like `One`)
     */
    data.variants.iter().all(|var| var.fields.is_empty())
}
fn enum_static_type(data: &DataEnum, repr: Option<Repr>, name: &Ident) -> Result<TokenStream, syn::Error> {
    let size = quote!(std::mem::size_of::<#name>());
    let equivalent_integer = match repr {
        Some(Repr::C) => {
            // Determine the equivalent unsigned integer representation
            quote!(static_reflect::types::IntType {
                size: static_reflect::types::IntSize::unwrap_from_bytes(#size),
                signed: false
            })
        },
        Some(Repr::Integer { bits, signed }) => {
            quote!(static_reflect::types::IntType {
                size: static_reflect::types::IntSize::unwrap_from_bytes(#bits as usize / 8),
                signed: #signed
            })
        },
        _ => return Err(syn::Error::new(name.span(), "Enum types must be either #[repr(C)] or #[repr(Int)]")),
    };
    if is_c_style_enum(data) {
        // C-style enum
        // TODO: Strict typing
        Ok(quote!(static_reflect::types::TypeInfo::Integer(#equivalent_integer)))
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "Complex enums are currently unsupported"
        ))
    }
}
trait TypeHandler<'a> {
    fn is_tuple_style(&self) -> bool;
    fn field_def_type(field_type: Option<TokenStream>) -> TokenStream;
    fn type_def_type() -> TokenStream;
    fn def_into_type(def_ref: TokenStream) -> TokenStream;
    fn handle_fields<F: FnMut(FieldInfo<'a>)>(&mut self, handler: F) -> syn::Result<()>;
    fn create_static_def(self, header: TokenStream) -> TokenStream;
}
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum FieldName<'a> {
    Tuple {
        index: usize,
    },
    Named {
        name: &'a Ident
    }
}
impl FieldName<'_> {
    pub fn access(&self) -> TokenStream {
        match *self {
            FieldName::Tuple { index } => {
                let idx = syn::Index::from(index);
                quote!(#idx)
            },
            FieldName::Named { name } => quote!(#name),
        }
    }
    pub fn associated_type_name(&self) -> Ident {
        match *self {
            FieldName::Tuple { index } => format_ident!("_Tuple_{}", index),
            FieldName::Named { name } => name.clone()
        }
    }
}
struct FieldInfo<'a> {
    name: FieldName<'a>,
    static_type: Type,
    static_def: TokenStream
}
struct StructHandler<'a> {
    name: &'a Ident,
    data: &'a DataStruct,
    current_offset: TokenStream
}
impl<'a> StructHandler<'a> {
    fn new(data: &'a DataStruct, name: &'a Ident) -> Self {
        StructHandler {
            name, data, current_offset: quote!(0)
        }
    }
}
impl<'a> TypeHandler<'a> for StructHandler<'a> {
    fn is_tuple_style(&self) -> bool {
        matches!(self.data.fields, syn::Fields::Unnamed(_))
    }

    fn field_def_type(field_type: Option<TokenStream>) -> TokenStream {
        match field_type {
            Some(inner) => quote!(static_reflect::types::FieldDef<'static, #inner>),
            None => quote!(static_reflect::types::FieldDef)
        }
    }

    fn type_def_type() -> TokenStream {
        quote!(static_reflect::types::StructureDef)
    }

    fn def_into_type(def_ref: TokenStream) -> TokenStream {
        quote!(static_reflect::types::TypeInfo::Structure(zerogc::epsilon::gc(#def_ref)))
    }

    fn handle_fields<F: FnMut(FieldInfo<'a>)>(&mut self, mut handler: F) -> syn::Result<()> {
        /*
         * NOTE: Layout algorithm for repr(C) given in reference
         * https://doc.rust-lang.org/reference/type-layout.html#reprc-structs
         * We have to use recursion to compute offsets :(
         */
        let mut current_offset = self.current_offset.clone();
        for (index, field) in self.data.fields.iter().enumerate() {
            let DeriveFieldOptions { opaque_array, assume_repr } =
                DeriveFieldOptions::parse_attrs(&field.attrs)?;
            let field_name = match field.ident {
                Some(ref name) => FieldName::Named { name },
                None => FieldName::Tuple { index }
            };
            let mut field_type = field.ty.clone();
            let original_type = field_type.clone();
            if opaque_array {
                if index + 1 != self.data.fields.len() {
                    return Err(syn::Error::new(
                        field.span(),
                        "Opaque array must be last field"
                    ));
                }
                match field_type.clone() {
                    Type::Array(array) => {
                        field_type = *array.elem;
                    },
                    _ => {
                        return Err(syn::Error::new(
                            field.span(),
                            "Type must be an array to be marked 'opaque_array'"
                        ))
                    }
                }
            }
            if let Some(assumed_type) = assume_repr {
                field_type = assumed_type;
            }
            /*
             * If the current offset is not a multiple of the field's alignment,
             * add the necessary padding bytes.
             */
            current_offset = quote!({
                let old_offset = #current_offset;
                /*
                 * NOTE: Must use #original_type instead of #field_type
                 * There is a chance an option like #[opaque_array] changed the size
                 */
                let rem = old_offset % std::mem::align_of::<#original_type>();
                old_offset + (if rem == 0 { 0 } else { std::mem::align_of::<#original_type>() - rem })
            });
            let name_field_value = match field_name {
                FieldName::Tuple { .. } => quote!(None),
                FieldName::Named { name } => quote!(Some(zerogc::epsilon::gc_str(stringify!(#name))))
            };
            let static_def = quote!(::static_reflect::types::FieldDef {
                name: #name_field_value,
                value_type: ::static_reflect::types::TypeId::<#field_type>::get(),
                offset: #current_offset,
                index: #index
            });
            handler(FieldInfo {
                name: field_name, static_type: field_type, static_def
            });
            // NOTE: Must use size_of<#original_type> (See above)
            current_offset = quote!((#current_offset) + std::mem::size_of::<#original_type>());
        }
        self.current_offset = current_offset;
        Ok(())
    }

    fn create_static_def(self, header: TokenStream) -> TokenStream {
        let name = self.name;
        let current_offset = &self.current_offset;
        quote!({
            use std::mem::{size_of, align_of};
            #header
            let def = StructureDef {
                name: zerogc::epsilon::gc_str(stringify!(#name)),
                fields: zerogc::epsilon::gc_array(_FIELDS),
                size: size_of::<#name>(),
                alignment: align_of::<#name>(),
            };
            let current_offset = #current_offset;
            let expected_size = current_offset + current_offset % align_of::<#name>();
            // In the case of zero-fields, default to alignment of `()`
            let mut expected_alignment = align_of::<()>();
            {
                // NOTE: Can't use for-loop since iterators aren't const
                let mut index = 0;
                let fields = _FIELDS;
                while index < fields.len() {
                    let alignment = fields[index].value_type.type_ref().value().alignment();
                    if alignment > expected_alignment {
                        expected_alignment = alignment;
                    }
                    index += 1;
                }
            }
            if def.size != expected_size {
                panic!("Mismatched size");
            }
            if def.alignment != expected_alignment {
                panic!("Mismatched alignments")
            }
            def
        })
    }
}
struct UnionTypeHandler<'a> {
    data: &'a DataUnion,
    name: &'a Ident
}
impl<'a> TypeHandler<'a> for UnionTypeHandler<'a> {
    fn is_tuple_style(&self) -> bool {
        false // unions can't have tuple-fields
    }

    fn field_def_type(field_type: Option<TokenStream>) -> TokenStream {
        match field_type {
            None => quote!(static_reflect::types::UnionFieldDef),
            Some(inner) => quote!(static_reflect::types::UnionFieldDef<'static, #inner>),
        }
    }

    fn type_def_type() -> TokenStream {
        quote!(static_reflect::types::UntaggedUnionDef)
    }

    fn def_into_type(def_ref: TokenStream) -> TokenStream {
        quote!(static_reflect::types::TypeInfo::UntaggedUnion(zerogc::epsilon::gc(#def_ref)))
    }

    fn handle_fields<F: FnMut(FieldInfo<'a>)>(&mut self, mut handler: F) -> syn::Result<()> {
        /*
         * NOTE: Layout algorithm for repr(C) given in reference
         * https://doc.rust-lang.org/reference/type-layout.html#reprc-unions
         *
         * Unions are pretty simple since they're just glorified `mem::transmute`
         */
        for (index, field) in self.data.fields.named.iter().enumerate() {
            let DeriveFieldOptions { opaque_array, assume_repr } =
                DeriveFieldOptions::parse_attrs(&field.attrs)?;
            if opaque_array {
                return Err(syn::Error::new(
                    field.span(),
                    "opaque_array is not supported on unions"
                ));
            }
            let field_name = field.ident.as_ref().expect("Need named fields");
            let mut field_type = field.ty.clone();
            if let Some(assumed_type) = assume_repr {
                field_type = assumed_type;
            }
            let static_def = quote!(::static_reflect::types::UnionFieldDef {
                name: zerogc::epsilon::gc_str(stringify!(#field_name)),
                value_type: ::static_reflect::types::TypeId::<#field_type>::get(),
                index: #index
            });
            handler(FieldInfo {
                name: FieldName::Named { name: field_name },
                static_type: field_type,
                static_def
            });
        }
        Ok(())
    }

    fn create_static_def(self, header: TokenStream) -> TokenStream {
        let name = self.name;
        quote!({
            use std::mem::{size_of, align_of};
            #header
            let def = UntaggedUnionDef {
                name: zerogc::epsilon::gc_str(stringify!(#name)),
                fields: zerogc::epsilon::gc_array(_FIELDS),
                size: size_of::<#name>(),
                alignment: align_of::<#name>(),
            };
            // In the case of zero-fields, default to alignment and size of `()`
            let mut expected_alignment = align_of::<()>();
            let mut expected_size = size_of::<()>();
            {
                // NOTE: Can't use for-loop since iterators aren't const
                let mut index = 0;
                while index < _FIELDS.len() {
                    let alignment = _FIELDS[index].value_type.type_ref().value().alignment();
                    let size = _FIELDS[index].value_type.type_ref().value().size();
                    if alignment > expected_alignment {
                        expected_alignment = alignment;
                    }
                    if size > expected_size {
                        expected_size = size;
                    }
                    index += 1;
                }
            }
            {
                // Round expected size up to next multiple of alignment
                let rem = expected_size % expected_alignment;
                if rem != 0 {
                    expected_size += expected_alignment - rem;
                }
            }
            if def.size != expected_size {
                panic!("Mismatched size");
            }
            if def.alignment != expected_alignment {
                panic!("Mismatched alignments")
            }
            def
        })
    }
}

fn add_type_bounds(generics: &Generics, bounds: &[TypeParamBound]) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.extend(bounds.iter().cloned());
        }
    }
    generics
}
