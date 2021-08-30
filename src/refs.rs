//! Provides a [TypeRef],
//! which can be used to allocate
use std::hash::Hash;
use std::fmt::Debug;
use std::ops::Deref;

use crate::{TypeInfo, types::{CStyleEnumDef, TaggedUnionVariant, CStyleEnumVariant, FieldDef, StructureDef, TaggedUnionDef, UnionFieldDef, UntaggedUnionDef}};

/// An allocator for [TypeInfo]
///
/// This abstracts over both statically
/// known `&'static T` references (the default)
/// but also supports other types of references
/// (ex. arena allocation or `zerogc`).
pub trait TypeAlloc: Sized {
    /// A reference to a TypeInfo (ex. &'static TypeInfo)
    type InfoRef: TypeRef<TypeInfo<Self>>;
    type InfoRefArray: TypeRef<[TypeInfo<Self>]>;
    type StructureDef: TypeRef<StructureDef<Self>>;
    type UntaggedUnionDef: TypeRef<UntaggedUnionDef<Self>>;
    type TaggedUnionDef: TypeRef<TaggedUnionDef<Self>>;
    type CStyleEnumDef: TypeRef<CStyleEnumDef<Self>>;
    type String: TypeRef<str>;
    type FieldDefArray: TypeRef<[FieldDef<(), Self>]>;
    type CStyleEnumVariantArray: TypeRef<[CStyleEnumVariant<Self>]>;
    type TaggedUnionVariantArray: TypeRef<[TaggedUnionVariant<Self>]>;
    type UnionFieldDefArray: TypeRef<[UnionFieldDef<(), Self>]>;
}
pub trait TypeRef<T: ?Sized>: Copy + Deref<Target=T> + Hash + Eq + PartialEq + Debug {}
impl<T: ?Sized + Eq + Debug + Hash + 'static> TypeRef<T> for &'static T {}

/// Indicates that a type has been statically allocated
/// and lives for `&'static T`
#[derive(Debug)]
pub struct StaticAlloc {
    _priv: ()
}
impl TypeAlloc for StaticAlloc {
    type InfoRef = &'static TypeInfo;

    type InfoRefArray = &'static [TypeInfo];

    type StructureDef = &'static StructureDef;

    type UntaggedUnionDef = &'static UntaggedUnionDef;

    type TaggedUnionDef = &'static TaggedUnionDef;

    type CStyleEnumDef = &'static CStyleEnumDef;

    type String = &'static str;

    type FieldDefArray = &'static [FieldDef];

    type CStyleEnumVariantArray = &'static [CStyleEnumVariant];

    type TaggedUnionVariantArray = &'static [TaggedUnionVariant];

    type UnionFieldDefArray = &'static [UnionFieldDef];
}