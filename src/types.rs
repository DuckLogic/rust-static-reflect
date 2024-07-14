//! The static type system
use crate::{StaticReflect, FieldReflect, PrimInt, PrimFloat};

#[cfg(feature = "num")]
pub use self::num::{PrimNum, PrimValue};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::fmt::{self, Formatter, Display, Debug, Write};

#[cfg(feature = "gc")]
use zerogc_derive::NullTrace;

#[cfg(feature = "builtins")]
use crate::builtins::{AsmSlice, AsmStr};
use std::alloc::Layout;

/// A type which is never zero, and where optional types
/// are guaranteed to use the null-pointer representation
///
/// If `T: SimpleNonZeroPointer` -> `sizeof(Option<T>) == sizeof(T) && repr(Option<T>) == repr(T)`
pub unsafe trait SimpleNonZeroRepr: StaticReflect {}

/// An integer size, named in the style of C/Java
///
/// Although named after their C equivalents,
/// they are not necessarily compatible.
/// For example, the C standard technically allows 16-bit ints
/// or 32-bit longs.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature="gc", derive(NullTrace))]
#[repr(u8)]
pub enum IntSize {
    /// A single byte (`u8`)
    Byte = 1,
    /// A two byte short (`u16`)
    Short = 2,
    /// A four byte integer (`u32`)
    ///
    /// This is the default integer type (by convention)
    Int = 4,
    /// An eight byte integer (`u64`)
    Long = 8
}
impl IntSize {
    /// Get the size of the specified primitive integer
    pub const fn of<T: PrimInt>() -> IntSize {
        T::INT_SIZE
    }
    /// A pointer-sized integer
    pub const POINTER: IntSize = {
        #[cfg(target_pointer_width = "16")] {
            IntSize::Short
        }
        #[cfg(target_pointer_width = "32")] {
            IntSize::Int
        }
        #[cfg(target_pointer_width = "64")] {
            IntSize::Long
        }
    };
    /// The size of the integer in bytes
    #[inline]
    pub const fn bytes(self) -> usize {
        self as usize
    }
    /// Create a new integer with the specified number of bytes,
    /// panicking if it is invalid
    ///
    /// TODO: Remove when `Result::unwrap` becomes a const-fn
    #[inline]
    pub const fn unwrap_from_bytes(bytes: usize) -> IntSize {
        match Self::from_bytes(bytes) {
            Ok(res) => res,
            Err(_) => panic!("Invalid number of bytes")
        }
    }
    /// Get an integer size corresponding to the specified number of bytes
    #[inline]
    pub const fn from_bytes(bytes: usize) -> Result<IntSize, InvalidSizeErr> {
        Ok(match bytes {
            1 => IntSize::Byte,
            2 => IntSize::Short,
            4 => IntSize::Int,
            8 => IntSize::Long,
            _ => return Err(InvalidSizeErr { bytes })
        })
    }
    /// Create an unsigned [IntType] with this size
    #[inline]
    pub const fn unsigned(self) -> IntType {
        IntType {
            size: self,
            signed: false
        }
    }
    /// Create an signed [IntType] with this size
    #[inline]
    pub const fn signed(self) -> IntType {
        IntType {
            size: self,
            signed: true
        }
    }
}
impl Default for IntSize {
    #[inline]
    fn default() -> IntSize {
        IntSize::Int // `int` is the conventional default
    }
}
/// An error indicating that the size is invalid
#[derive(Debug)]
pub struct InvalidSizeErr {
    /// The size in bytes that is considered invalid
    pub bytes: usize,
}
impl Display for InvalidSizeErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Invalid size: {}", self.bytes)
    }
}
impl std::error::Error for InvalidSizeErr {}

/// The size of a floating point number,
/// either single-precision or double-precision
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub enum FloatSize {
    /// A single-precision floating point number.
    ///
    /// This type is 4 bytes (`f32`)
    Single = 4,
    /// A double-precision floating point number.
    /// 
    /// By convention, this type is the default.
    /// This type is 8 bytes (`f64`).
    Double = 8
}
impl FloatSize {
    /// Get the size of the specified float
    #[inline]
    pub const fn of<T: PrimFloat>() -> FloatSize {
        T::FLOAT_SIZE
    }
    /// The number of bytes for a float of this size
    #[inline]
    pub const fn bytes(self) -> usize {
        self as usize
    }
    /// Get a [FloatSize] corresponding to the specified
    /// number of bytes.
    #[inline]
    pub const fn from_bytes(bytes: usize) -> Result<FloatSize, InvalidSizeErr> {
        Ok(match bytes {
            4 => FloatSize::Single,
            8 => FloatSize::Double,
            _ => return Err(InvalidSizeErr { bytes })
        })
    }
}
impl Default for FloatSize {
    #[inline]
    fn default() -> FloatSize {
        FloatSize::Double
    }
}

/// An integer type
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub struct IntType {
    /// The size of this integer
    pub size: IntSize,
    /// If this integer is signed
    pub signed: bool,
}
impl IntType {
    /// Get the type of the specified primitive integer
    #[inline]
    pub const fn of<T: PrimInt>() -> IntType {
        T::INT_TYPE
    }
    /// The alignment of this integer
    #[inline]
    pub const fn align(&self) -> usize {
        use std::mem::align_of;
        match *self {
            IntType { size: IntSize::Byte, signed: false } => align_of::<u8>(),
            IntType { size: IntSize::Short, signed: false } => align_of::<u16>(),
            IntType { size: IntSize::Int, signed: false } => align_of::<u32>(),
            IntType { size: IntSize::Long, signed: false } => align_of::<u64>(),
            IntType { size: IntSize::Byte, signed: true } => align_of::<i8>(),
            IntType { size: IntSize::Short, signed: true } => align_of::<i16>(),
            IntType { size: IntSize::Int, signed: true } => align_of::<i32>(),
            IntType { size: IntSize::Long, signed: true } => align_of::<i64>(),
        }
    }
    /// The type of the unsigned `u8` integer
    pub const U8: IntType = IntSize::Byte.unsigned();
    /// The type of the unsigned `u16` integer
    pub const U16: IntType = IntSize::Short.unsigned();
    /// The type of the unsigned `u32` integer
    pub const U32: IntType = IntSize::Int.unsigned();
    /// The type of the unsigned `u64` integer
    pub const U64: IntType = IntSize::Long.unsigned();
    /// The type of the unsigned `usize` integer
    pub const USIZE: IntType = IntSize::POINTER.unsigned();
    /// The type of the signed `i8` integer
    pub const I8: IntType = IntSize::Byte.signed();
    /// The type of the signed `i16` integer
    pub const I16: IntType = IntSize::Short.signed();
    /// The type of the signed `i32` integer
    pub const I32: IntType = IntSize::Int.signed();
    /// The type of the signed `i64` integer
    pub const I64: IntType = IntSize::Long.signed();
    /// The type of the signed `isize` integer
    pub const ISIZE: IntType = IntSize::POINTER.signed();
}
impl Display for IntType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char(if self.signed { 'i' } else { 'u' })?;
        write!(f, "{}", self.size.bytes() * 8)?;
        Ok(())
    }
}

/// Represents the different styles of FFI-compatible tagged unions.
///
/// According to [RFC #2195](https://github.com/rust-lang/rfcs/blob/master/text/2195-really-tagged-unions.md),
/// there are two different representations for tagged unions in Rust:
/// 1. The ["traditional" representation](https://doc.rust-lang.org/stable/reference/type-layout.html#reprc-enums-with-fields) (`#[repr(C)]`)
/// 2. The ["primitive" representation](https://doc.rust-lang.org/stable/reference/type-layout.html#primitive-representation-of-enums-with-fields)] (`#[repr(u8)]`, `#[repr(isize)]`, etc...)
///
/// ## Traditional representation (default)
/// The "traditional" representation is the most intuitive representation of tagged unions, and is
/// what you would expect coming from a C/C++ background. It is the default if `#[repr(C)]` is specified:
///
/// ````no_run
/// #[repr(C)]
/// enum Traditional {
///     One(u8, u16),
///     Two(u8)
/// }
/// /* ---- is equivalent to ---- */
/// #[repr(C)]
/// enum TraditionalTag {
///     One,
///     Two
/// }
/// #[repr(C)]
/// union TraditionalData {
///     one: (u8, u16),
///     two: u8
/// }
/// #[repr(C)]
/// struct TraditionalRepr {
///     tag: TraditionalTag,
///     data: TraditionalData,
/// }
/// ````
/// This appears all fine and dandy, until you realize that the `u16` in the first variant needs 16-bit
/// alignment. This means that `TraditionalData.one` needs a padding byte between the `u8` and the `u16`,
/// making `mem::size_of::<TraditionalData>() == 4`, even though it only has 3 bytes of meaningful dataa.
///
/// Since that `TraditionalTag` is *separate* from `TraditionalData`,
/// this makes `mem::size_of::<TraditionalRepr>() == 5` (even though it only has 4 bytes of meaningful data):
/// `[<tag>, u8, <wasted padding>, u16]`
///
/// Despite this wasted space, this is the default layout for `#[repr(C)]` enums,
/// because it's easier to use with C-code.
///
/// ## The "primitive" representation
/// The "primitive" representation of tagged enums have the same
/// names as the primitive integer types. For example `#[repr(u8)]`, `#[repr(isize)]`, etc..
///
/// See the [reference](https://doc.rust-lang.org/stable/reference/type-layout.html#primitive-representation-of-enums-with-fields)
/// for the official documentation on this representation.
///
/// This layout is more "efficient" than the 'traditional' representation,
/// by specifying `#[repr(u8)]` (or some other specific variant type).
///
/// It represents a Rust enum as a union of structs,
/// where each individual sub-struct starts with the tag enum.
///
/// This avoids any wasted padding bytes, but is slightly harder to use and seems
/// unexpected if you come from a C/C++ background.
/// ````no_run
/// use std::mem::ManuallyDrop;
/// #[repr(u8)]
/// enum Efficient {
///     One(u8, u16),
///     Two(u16)
/// }
/// #[repr(u8)]
/// enum EfficientTag {
///     One,
///     Two
/// }
/// #[repr(C)]
/// union EfficientRepr {
///     one: ManuallyDrop<EfficientVariantOne>,
///     two: ManuallyDrop<EfficientVariantTwo>
/// }
/// #[repr(C)]
/// struct EfficientVariantOne {
///     tag: EfficientTag,
///     first: u8,
///     second: u16
/// }
/// #[repr(C)]
/// struct EfficientVariantTwo {
///     tag: EfficientTag,
///     data: u16,
/// }
/// ````
/// As you can see, there is no need for padding bytes in `EfficientVariantOne`.
/// The `second` field is naturally aligned, making the whole `EfficientRepr` only `4` bytes,
/// in contrast to the 5-byte representation of `TraditionalRepr`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub enum TaggedUnionStyle {
    /// The ["traditional" representation](https://doc.rust-lang.org/nightly/reference/type-layout.html#reprc-enums-with-fields),
    /// which is specified by `#[repr(C)]`
    ///
    /// Essentially, it stores tagged enums as a combination of `(tag, union { data })`.
    ///
    /// This is the most intuitive representation, and is what you probably
    /// expect if you come from a C/C++ background,
    /// and is the default if you specify `#[repr(C)]`.
    Traditional,
    /// The more efficient ["primitive" representation](https://doc.rust-lang.org/nightly/reference/type-layout.html#reprc-enums-with-fields)
    /// which is specified by `#[repr(u8)]`, `#[repr(isize)]`, etc...
    ///
    /// This stores tagged enums as a union, with the tag as the first field of each variant.
    /// In some cases, this can be more efficient than the "traditional" representation.
    Primitive
}
impl TaggedUnionStyle {
    /// Compute the `Layout` of an enum with this style and the specified
    /// discriminant and variant layouts.
    ///
    /// Panics if the enum is uninhabited, or an error occurs calculating the combined layouts..
    pub fn compute_layout(&self, discriminant_size: Layout, variant_layouts: impl Iterator<Item=Layout>) -> Layout {
        let mut starting_layout = discriminant_size.clone();
        match *self {
            TaggedUnionStyle::Traditional => {
                /*
                 * this is what makes us different from the "primitive" repr.
                 * We have padding before the start of each variant.
                 */
                starting_layout = starting_layout.pad_to_align();
            },
            TaggedUnionStyle::Primitive => {
                // We're more efficient - no padding before the start of each variant
            }
        }
        let mut max_size = None;
        let mut max_alignment = starting_layout.align();
        for variant_layout in variant_layouts {
            let (combined_layout, _) = starting_layout.extend(variant_layout).unwrap();
            max_size = Some(max_size.unwrap_or(0).max(combined_layout.size()));
            max_alignment = max_alignment.max(combined_layout.align());
        }
        let size = max_size.expect("Uninhabited enum");
        Layout::from_size_align(size, max_alignment).unwrap()
    }
}
impl Default for TaggedUnionStyle {
    #[inline]
    fn default() -> Self {
        TaggedUnionStyle::Traditional
    }
}

/// A type whose representation is known via reflection
///
/// These are usually defined statically via [StaticReflect
///
/// However, they can be allocated at runtime,
/// and potentially live for a more limited lifetime.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub enum TypeInfo {
    /// The zero-length unit type `()`
    ///
    /// Used for functions that return nothing
    Unit,
    /// An impossible type,
    ///
    /// The mere existence of this type at runtime is undefined behavior.
    /// Functions that have this as their `return` type never actually return.
    #[cfg(feature = "never")]
    Never,
    /// A boolean
    ///
    /// Like a Rust `bool`, values must be either zero or one.
    /// Anything else is undefined behavior.
    Bool,
    /// An integer
    Integer(IntType),
    /// A floating point number
    Float {
        /// The size/precision of the float
        size: FloatSize
    },
    /// A slice of memory, represented as pointer + length
    ///
    /// The element type is needed, since array indexing implicitly
    /// multiples by the size of the memory.
    ///
    /// Representation should match the [AsmSlice] type
    #[cfg(feature = "builtins")]
    Slice {
        /// The type of the inner element
        element_type: &'static TypeInfo,
    },
    /// A pointer to a UTF8 encoded string and length,
    /// just like Rust's 'str' type
    ///
    /// Internally represented by the [AsmStr] structure
    #[cfg(feature = "builtins")]
    Str,
    /// A very simple optional, represented as an [AsmOption](crate::builtins::AsmOption)
    ///
    /// This **never** uses the null pointer optimization
    #[cfg(feature = "builtins")]
    Optional(&'static TypeInfo),
    /// An untyped pointer
    ///
    /// This may be null.
    ///
    /// Untyped pointers simplify the type system significantly.
    /// They also avoid cycles when defining structures
    /// in case a structure contains a pointer to itself.
    Pointer,
    /// A structure
    Structure(&'static StructureDef),
    /// An untagged union
    UntaggedUnion(&'static UntaggedUnionDef),
    /// A tagged union with a well-defined Rust-compatible layout.
    /// See RFC #2195 for complete details on how `#[repr(C)]` enums are defined.
    ///
    /// There are two different representations for tagged unions.
    /// See [TaggedUnionStyle] for details.
    TaggedUnion(&'static TaggedUnionDef),
    /// A C-style enum, without any data.
    ///
    /// See [TypeInfo::TaggedUnion] for enums *with* data.
    CStyleEnum(&'static CStyleEnumDef),
    /// A named, transparent, extern type
    Extern {
        /// The name of the type
        ///
        /// Since this is all we have, it's what used
        /// to disambiguate between them.
        name: &'static str
    },
    /// A 'magic' type, with a user-defined meaning
    ///
    /// This allows extensions to the type system
    Magic {
        /// The id of the magic type,
        /// giving more information about how its implemented
        /// and what it actually means.
        id: &'static &'static str,
        /// Extra information (if any)
        extra: Option<&'static TypeInfo>
    }
}
impl TypeInfo {
    /// A 32-bit, single-precision float
    pub const F32: Self = TypeInfo::Float { size: FloatSize::Single };
    /// A 64-bit, double-precision float
    pub const F64: Self = TypeInfo::Float { size: FloatSize::Double };
}
impl TypeInfo {
    /// The size of the type, in bytes
    pub const fn size(&self) -> usize {
        use std::mem::size_of;
        use self::TypeInfo::*;
        match *self {
            Unit => 0,
            #[cfg(feature = "never")]
            Never => size_of::<!>(),
            Bool => size_of::<bool>(),
            Integer(IntType { size, .. }) => size.bytes(),
            Float { size } => size.bytes(),
            #[cfg(feature = "builtins")]
            Slice { .. } => std::mem::size_of::<AsmSlice<()>>(),
            #[cfg(feature = "builtins")]
            Optional(ref _inner) => unimplemented!(),
            Pointer => size_of::<*const ()>(),
            #[cfg(feature = "builtins")]
            Str => size_of::<AsmStr>(),
            Structure(ref def) => def.size,
            UntaggedUnion(ref def) => def.size,
            TaggedUnion(def) => def.size,
            CStyleEnum(def) => def.discriminant.size.bytes(),
            // Provide a dummy value
            TypeInfo::Magic { .. } | TypeInfo::Extern { .. } => 0xFFFF_FFFF
        }
    }
    /// The alignment of the type, matching `std::mem::align_of`
    pub const fn alignment(&self) -> usize {
        use std::mem::align_of;
        match *self {
            TypeInfo::Unit => align_of::<()>(),
            #[cfg(feature = "never")]
            TypeInfo::Never => align_of::<!>(),
            TypeInfo::Magic { .. } | TypeInfo::Extern { .. } => 0,
            TypeInfo::Bool => align_of::<bool>(),
            TypeInfo::Integer(tp) => tp.align(),
            TypeInfo::Float { size: FloatSize::Single } => align_of::<f32>(),
            TypeInfo::Float { size: FloatSize::Double } => align_of::<f64>(),
            #[cfg(feature = "builtins")]
            TypeInfo::Slice { .. } | TypeInfo::Optional(_) => unimplemented!(),
            TypeInfo::Pointer => align_of::<*const ()>(),
            #[cfg(feature = "builtins")]
            TypeInfo::Str => align_of::<AsmStr>(),
            TypeInfo::Structure(def) => def.alignment,
            TypeInfo::UntaggedUnion(def) => def.alignment,
            TypeInfo::CStyleEnum(def) => def.discriminant.align(),
            TypeInfo::TaggedUnion(def) => def.alignment
        }
    }
}
impl Display for TypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            TypeInfo::Unit => f.write_str("()"),
            TypeInfo::Never => f.write_str("!"),
            TypeInfo::Bool => f.write_str("bool"),
            TypeInfo::Integer(tp) => write!(f, "{}", tp),
            TypeInfo::Float { size } => write!(f, "f{}", size.bytes() * 8),
            TypeInfo::Slice { element_type } => write!(f, "[{}]", element_type),
            TypeInfo::Str => f.write_str("str"),
            TypeInfo::Optional(inner_type) => write!(f, "Option<{}>", inner_type),
            TypeInfo::Pointer => f.write_str("*mut void"),
            TypeInfo::Structure(ref def) => f.write_str(def.name),
            TypeInfo::UntaggedUnion(ref def) => f.write_str(def.name),
            TypeInfo::CStyleEnum(ref def) => f.write_str(def.name),
            TypeInfo::TaggedUnion(ref def) => f.write_str(def.name),
            TypeInfo::Extern { name } => write!(f, "extern {}", name),
            TypeInfo::Magic { id, extra: None } => write!(f, "magic::{}", id),
            TypeInfo::Magic { id, extra: Some(extra) } => write!(f, "magic::{}<{}>", id, extra)
        }
    }
}
/// Static information on the definition of a structure
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub struct StructureDef {
    /// The name of the structure
    pub name: &'static str,
    /// All of the fields defined in the structure
    pub fields: &'static [FieldDef],
    /// The total size of the structure (including padding)
    pub size: usize,
    /// The required alignment of the structure
    pub alignment: usize,
}
impl<T: StaticReflect> Copy for FieldDef<T> {}
impl<T: StaticReflect> Clone for FieldDef<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
/// The definition of a field
#[derive(Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
#[cfg_attr(feature="gc",zerogc(ignore_params(T)))]
pub struct FieldDef<T: StaticReflect = ()> {
    /// The name of the field, or `None` if this is a tuple struct
    pub name: Option<&'static str>,
    /// The type of the field
    pub value_type: TypeId<T>,
    /// The offset of the field in bytes
    pub offset: usize,
    /// The numeric index of the field
    ///
    /// Should correspond to the order of declaration
    pub index: usize
}
impl<T: StaticReflect> FieldDef<T> {
    /// Erase the static type information from this field definition
    #[inline]
    pub const fn erase(&self) -> FieldDef {
        FieldDef {
            name: self.name,
            value_type: self.value_type.erase(),
            offset: self.offset,
            index: self.index
        }
    }
    /// The offset of the field, in bytes
    #[inline]
    pub const fn offset(&self) -> usize {
        self.offset
    }
}
/// The definition of C-style enum
///
/// The variants of a C-style enum may not have any data.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub struct CStyleEnumDef {
    /// The name of the enumeration
    pub name: &'static str,
    /// The integer type of the discriminant
    ///
    /// This is what determines the enum's runtime size and alignment.
    pub discriminant: IntType,
    /// The valid variants of this enum
    pub variants: &'static [CStyleEnumVariant]
}
impl CStyleEnumDef {
    /// Determines whether this enum has any explicit discriminant values,
    /// overriding the defaults.
    ///
    /// If this is `false`, then the value of each variant's discriminant
    /// is implicitly equal to its index
    #[inline]
    pub fn has_explicit_discriminants(&self) -> bool {
        self.variants.iter().any(|variant| variant.discriminant.is_explicit())
    }
}
/// A variant in a C-style enum (a Rust enum without any data)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub struct CStyleEnumVariant {
    /// The index of this variant, specifying the declaration order
    pub index: usize,
    /// The name of this variant
    pub name: &'static str,
    /// The value of the enum's discriminant
    pub discriminant: DiscriminantValue
}
/// The value of the discriminant
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub enum DiscriminantValue {
    /// The discriminant has the default value,
    /// which is implicitly equal to its declaration order.
    Default {
        /// The index
        declaration_index: usize
    },
    /// This discriminant hasn't been explicitly specified,
    ///
    /// However, a previous discriminant *has* been explicitly specified,
    /// and is implicitly offsetting future values.
    ImplicitlyOffset {
        /// The raw bits of the discriminant,
        /// appropriately offset by previous declarations
        bits: u64,
    },
    /// The discriminant has been specified explicitly
    ///
    /// This
    ExplicitInteger {
        /// The raw bits of the explicit discriminant's value.
        ///
        /// It is possible that this is a negative value, depending on the [IntType] of the discriminant.
        bits: u64
    },
}
impl DiscriminantValue {
    /// Whether this discriminant has been specified explicitly
    #[inline]
    pub fn is_explicit(&self) -> bool {
        matches!(*self, DiscriminantValue::ExplicitInteger { .. })
    }
    /// The bits of the discriminant.
    ///
    /// Depending on the [IntType] of the discriminant,
    /// it is possible this is a negative value (even though the static type is `u64`)
    #[inline]
    pub fn bits(&self) -> u64 {
        match *self {
            DiscriminantValue::Default { declaration_index } => declaration_index as u64,
            DiscriminantValue::ImplicitlyOffset { bits } |
            DiscriminantValue::ExplicitInteger { bits } => bits
        }
    }
}
/// The definition of a FFI-compatible enum with data.
///
/// These are just FFI-compatible Rust enums annotated with `#[repr(C)]`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub struct TaggedUnionDef {
    /// The name of the enum type
    pub name: &'static str,
    /// The "style" of the tagged union.
    ///
    /// Tagged unions have two possible representations.
    /// See [TaggedUnionStyle] docs for more info.
    pub style: TaggedUnionStyle,
    /// The type of the enum's discriminant
    pub discriminant_type: IntType,
    /// The variants of this tagged enum
    pub variants: &'static [TaggedUnionVariant],
    /// The size of the type
    pub size: usize,
    /// The alignment of the type
    ///
    /// This should be equal to max(discriminant.align, max(variant.align for variant in variants))
    pub alignment: usize
}

/// A variant in a tagged union (Rust-style enum)
///
/// This mostly functions as a wrapper around a [StructureDef],
/// which stores information on the variant's fields (and whether or
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub struct TaggedUnionVariant {
    /// The index of this variant, determining the declaration order
    pub index: usize,
    /// The structure this enum-variant is equivalent to.
    ///
    /// It has a matching name, fields, and size.
    pub equivalent_structure: StructureDef,
    /// The value of the enum's discriminant
    pub discriminant: DiscriminantValue
}
impl TaggedUnionVariant {
    /// The name of the variant
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.equivalent_structure.name
    }
}
/// The definition of an untagged union which is known at compile-time
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub struct UntaggedUnionDef {
    /// The name of the union
    pub name: &'static str,
    /// The fields of the union
    pub fields: &'static [UnionFieldDef],
    /// The size of the union, in bytes
    ///
    /// Should equal the size of its largest member
    pub size: usize,
    /// The required alignment of the union, in bytes
    ///
    /// I believe this should equal the maximum
    /// of the alignments required alignment by its members
    pub alignment: usize,
}

/// A field of a union which is known at compile-time
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
#[cfg_attr(feature="gc",zerogc(ignore_params(T)))]
pub struct UnionFieldDef<T: StaticReflect = ()> {
    /// The name of the field
    pub name: &'static str,
    /// The type of the field
    pub value_type: TypeId<T>,
    /// The numeric index of the field
    ///
    /// This has no effect on generated code, but it should probably correspond
    /// to the order of declaration in the source code
    pub index: usize
}
impl<T: StaticReflect> UnionFieldDef<T> {
    /// Erase the generic type of this field
    pub const fn erase(&self) -> UnionFieldDef {
        UnionFieldDef {
            name: self.name,
            value_type: self.value_type.erase(),
            index: self.index
        }
    }
    /// Offset of the field in the union
    ///
    /// The fields of unions never have any offset,
    /// so this is always zero.
    #[inline]
    pub const fn offset(&self) -> usize {
        0
    }
}

/// A primitive type
///
/// Although rust doesn't truly have a concept of 'primitives',
/// these are the most basic types needed to construct all the others.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature="gc", derive(NullTrace))]
pub enum PrimitiveType {
    /// The zero-length type '()'
    ///
    /// This is zero-length in the Rust sense,
    /// not the C sense
    Unit,
    /// The type for functions/instructions that can never return or occur
    ///
    /// It is undefined behavior for this type to even exist
    #[cfg(feature = "never")]
    Never,
    /// A boolean type, corresponding to Rust's [bool] type
    Bool,
    /// An untyped pointer (which is possibly null)
    Pointer,
    /// An integer
    Integer(IntType),
    /// A float
    Float {
        /// The size/precision of the float
        size: FloatSize
    },
}
impl PrimitiveType {
    /// The type information for this primitive type
    pub fn type_info(&self) -> &TypeInfo {
        use self::PrimitiveType::*;
        use self::IntSize::*;
        use self::FloatSize::*;
        match *self {
            Unit => &TypeInfo::Unit,
            Never => &TypeInfo::Never,
            Bool => &TypeInfo::Bool,
            Pointer => &TypeInfo::Pointer,
            Integer(IntType { size: Byte, signed: true }) => &TypeInfo::Integer(IntType::U8),
            Integer(IntType { size: Short, signed: true }) => &TypeInfo::Integer(IntType::U16),
            Integer(IntType  { size: Int, signed: true }) => &TypeInfo::Integer(IntType::U32),
            Integer(IntType { size: Long, signed: true }) => &TypeInfo::Integer(IntType::U64),
            Integer(IntType { size: Byte, signed: false }) => &TypeInfo::Integer(IntType::I8),
            Integer(IntType { size: Short, signed: false }) => &TypeInfo::Integer(IntType::I16),
            Integer(IntType { size: Int, signed: false }) => &TypeInfo::Integer(IntType::I32),
            Integer(IntType { size: Long, signed: false }) => &TypeInfo::Integer(IntType::I64),
            Float { size: Single } => &TypeInfo::Float { size: Single },
            Float { size: Double } => &TypeInfo::Float { size: Double },
        }
    }

    /// The number of bytes this type tales up
    pub fn bytes(&self) -> usize {
        match self {
            PrimitiveType::Unit | PrimitiveType::Never => 0,
            PrimitiveType::Integer(tp) => tp.size.bytes(),
            PrimitiveType::Float { size } => size.bytes(),
            PrimitiveType::Pointer => {
                assert_eq!(std::mem::size_of::<usize>(), std::mem::size_of::<*mut ()>());
                std::mem::size_of::<*mut ()>() as usize
            },
            PrimitiveType::Bool => {
                assert_eq!(std::mem::size_of::<bool>(), 1);
                1
            },
        }
    }
    /// The size of this type, in bytes
    #[inline]
    pub fn size(self) -> usize {
        self.bytes() as usize
    }
}
/// Compare two primitive types based on their sizes
impl PartialOrd for PrimitiveType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (*self, *other) {
            (
                PrimitiveType::Integer(IntType { size: first, signed: first_signed }),
                PrimitiveType::Integer(IntType { size: second, signed: second_signed })
            ) => {
                Some(first.cmp(&second)
                    .then(first_signed.cmp(&second_signed)))
            },
            (PrimitiveType::Float { size: first }, PrimitiveType::Float { size: second }) => Some(first.cmp(&second)),
            (first, other) if first == other => Some(Ordering::Equal),
            _ => None
        }
    }
}

impl<T: StaticReflect> Copy for TypeId<T> {}
impl<T: StaticReflect> Clone for TypeId<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// An static reference to a type
#[derive(Eq, PartialEq, Hash)]
#[cfg_attr(feature="gc", derive(NullTrace))]
#[cfg_attr(feature="gc",zerogc(ignore_params(T)))]
pub struct TypeId<T: StaticReflect = ()> {
    value: &'static TypeInfo,
    marker: PhantomData<fn() -> T>
}
impl TypeId {
    /// Get the erased TypeId of the specified type `T`
    #[inline]
    pub const fn erased<T: StaticReflect>() -> TypeId {
        TypeId::<T>::get().erase()
    }
}
impl<T: StaticReflect> TypeId<T> {
    /// Get the TypeId of the corresponding (generic) type
    #[inline]
    pub const fn get() -> Self {
        TypeId {
            value: &T::TYPE_INFO,
            marker: PhantomData
        }
    }
    /// Return a checked [TypeId] corresponding to the specified
    #[inline]
    pub const fn from_static(s: &'static TypeInfo) -> Self {
        TypeId {
            value: s,
            marker: PhantomData
        }
    }
    /// Erase this type id,
    /// ignoring its statically-known generic
    /// parameters
    ///
    /// The generic parameters are unchecked, but are
    /// very useful for ensuring safety.
    #[inline]
    pub const fn erase(self) -> TypeId {
        TypeId {
            value: self.value,
            marker: PhantomData,
        }
    }
    /// If this type is a boolean
    #[inline]
    pub fn is_bool(self) -> bool {
        matches!(self.primitive(), Some(PrimitiveType::Bool))
    }
    /// If this type is an integer (of any size)
    #[inline]
    pub fn is_int(self) -> bool {
        matches!(self.primitive(), Some(PrimitiveType::Integer { .. }))
    }
    /// If this type is a pointer
    #[inline]
    pub fn is_ptr(self) -> bool {
        matches!(self.primitive(), Some(PrimitiveType::Pointer { .. }))
    }
    /// If this type is a floating point number (of any size)
    #[inline]
    pub fn is_float(self) -> bool {
        matches!(self.primitive(), Some(PrimitiveType::Float { .. }))
    }
    /// If this type is a primitive
    #[inline]
    pub fn is_primitive(self) -> bool {
        self.primitive().is_some()
    }
    /// Convert this type into its corresponding [PrimitiveType],
    /// or `None` if it's not a primitive.
    #[inline]
    pub fn primitive(self) -> Option<PrimitiveType> {
        Some(match *self.value {
            TypeInfo::Unit => PrimitiveType::Unit,
            TypeInfo::Never => PrimitiveType::Never,
            TypeInfo::Bool => PrimitiveType::Bool,
            TypeInfo::Pointer => PrimitiveType::Pointer,
            TypeInfo::Integer(tp) => PrimitiveType::Integer(tp),
            TypeInfo::Float { size } => PrimitiveType::Float { size },
            _ => return None
        })
    }
    /// A reference to the underlying type
    #[inline]
    pub const fn type_ref(self) -> &'static TypeInfo {
        self.value
    }
    /// Create a [TypeId] from the specified reference
    #[inline]
    #[deprecated(note = "Use from_")]
    pub const fn from_ref(tp: &'static TypeInfo) -> Self {
        TypeId {
            marker: PhantomData,
            value: tp
        }
    }
    /// The information on this type's named fields
    #[inline]
    pub const fn named_field_info(&self) -> <T as FieldReflect>::NamedFieldInfo
        where T: FieldReflect {
        T::NAMED_FIELD_INFO
    }

}
impl<T: StaticReflect> TypeId<*mut T> {
    /// The target of the pointer type
    ///
    /// NOTE: This relies on static typing information
    #[inline]
    pub const fn pointer_target(self) -> TypeId<T> {
        TypeId::get()
    }
}
impl<T: StaticReflect> From<&'static TypeInfo> for TypeId<T> {
    #[inline]
    fn from(static_type: &'static TypeInfo) -> Self {
        TypeId::from_static(static_type)
    }
}
impl<T: StaticReflect> Display for TypeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}
impl<T: StaticReflect> Debug for TypeId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypeId")
            .field(self.value)
            .finish()
    }
}

/// A indexed identifier of a field
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FieldId {
    /// The owner of the field
    pub owner: TypeId,
    /// The index of the field
    pub index: usize,
}

