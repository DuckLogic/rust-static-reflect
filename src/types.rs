//! The static type system
use crate::{StaticReflect, FieldReflect, PrimInt, PrimFloat};

#[cfg(feature = "num")]
pub use self::num::{PrimNum, PrimValue};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::fmt::{self, Formatter, Display, Debug, Write};

#[cfg(feature = "gc")]
use zerogc_derive::{unsafe_gc_impl};

#[cfg(feature = "builtins")]
use crate::builtins::{AsmSlice, AsmStr};

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

/// A type whose representation is known via reflection
///
/// These are usually defined statically via [StaticReflect
///
/// However, they can be allocated at runtime,
/// and potentially live for a more limited lifetime.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TypeInfo<'a> {
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
        element_type: &'a TypeInfo<'a>,
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
    Optional(&'a TypeInfo<'a>),
    /// An untyped pointer
    ///
    /// This may be null.
    ///
    /// Untyped pointers simplify the type system significantly.
    /// They also avoid cycles when defining structures
    /// in case a structure contains a pointer to itself.
    Pointer,
    /// A structure
    Structure(&'a StructureDef<'a>),
    /// An untagged union
    Union(&'a UnionDef<'a>),
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
        extra: Option<&'a TypeInfo<'a>>,
    }
}
/*
 * HACK: Implement AsmType as `NullTrace`
 *
 * Unfortunately this means the type cannot use
 * garbage collected references.
 */
#[cfg(feature = "gc")]
unsafe_gc_impl! {
    target => TypeInfo<'a>,
    params => ['a],
    bounds => {
        Trace => always,
        TraceImmutable => always,
        GcSafe => always,
        GcRebrand => { where 'a: 'new_gc },
        GcErase => { where 'a: 'min }
    },
    null_trace => always,
    branded_type => Self,
    erased_type => Self,
    NEEDS_TRACE => false,
    NEEDS_DROP => ::std::mem::needs_drop::<Self>(),
    visit => |self, visitor| { Ok(()) /* nop */ }
}
impl TypeInfo<'static> {
    /// A 32-bit, single-precision float
    pub const F32: Self = TypeInfo::Float { size: FloatSize::Single };
    /// A 64-bit, double-precision float
    pub const F64: Self = TypeInfo::Float { size: FloatSize::Double };
}
impl<'tp> TypeInfo<'tp> {
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
            Union(ref def) => def.size,
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
            TypeInfo::Structure(ref def) => def.alignment,
            TypeInfo::Union(ref def) => def.alignment,
        }
    }
}
impl<'a> Display for TypeInfo<'a> {
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
            TypeInfo::Union(ref def) => f.write_str(def.name),
            TypeInfo::Extern { name } => write!(f, "extern {}", name),
            TypeInfo::Magic { id, extra: None } => write!(f, "magic::{}", id),
            TypeInfo::Magic { id, extra: Some(extra) } => write!(f, "magic::{}<{}>", id, extra)
        }
    }
}
/// Static information on the definition of a structure
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct StructureDef<'a> {
    /// The name of the structure
    pub name: &'a str,
    /// All of the fields defined in the structure
    pub fields: &'a [FieldDef<'a>],
    /// The total size of the structure (including padding)
    pub size: usize,
    /// The required alignment of the structure
    pub alignment: usize,
}
impl<T: StaticReflect> Copy for FieldDef<'_, T> {}
impl<T: StaticReflect> Clone for FieldDef<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
/// The definition of a field
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct FieldDef<'tp, T: StaticReflect = ()> {
    /// The name of the field, or `None` if this is a tuple struct
    pub name: Option<&'tp str>,
    /// The type of the field
    pub value_type: TypeId<'tp, T>,
    /// The offset of the field in bytes
    pub offset: usize,
    /// The numeric index of the field
    ///
    /// Should correspond to the order of declaration
    pub index: usize
}
impl<'a, T: StaticReflect> FieldDef<'a, T> {
    /// Erase the static type information from this field definition
    #[inline]
    pub const fn erase(&self) -> FieldDef<'a> {
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
/// A `UnionDef` which is known at compile-time
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct UnionDef<'a> {
    /// The name of the union
    pub name: &'a str,
    /// The fields of the union
    pub fields: &'a [UnionFieldDef<'a>],
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
pub struct UnionFieldDef<'tp, T: StaticReflect = ()> {
    /// The name of the field
    pub name: &'tp str,
    /// The type of the field
    pub value_type: TypeId<'tp, T>,
    /// The numeric index of the field
    ///
    /// This has no effect on generated code, but it should probably correspond
    /// to the order of declaration in the source code
    pub index: usize
}
impl<'a, T: StaticReflect> UnionFieldDef<'a, T> {
    /// Erase the generic type of this field
    pub const fn erase(&self) -> UnionFieldDef<'a> {
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
    pub fn type_info(&self) -> &TypeInfo<'static> {
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

impl<T: StaticReflect> Copy for TypeId<'_, T> {}
impl<T: StaticReflect> Clone for TypeId<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// An static reference to a type
#[derive(Eq, PartialEq, Hash)]
pub struct TypeId<'a, T: StaticReflect = ()> {
    value: &'a TypeInfo<'a>,
    marker: PhantomData<fn() -> T>
}
/*
  * TODO: Fix to use derive
  *
  * Right now that doesn't work since it requires `T: 'a`
 */
#[cfg(feature = "gc")]
unsafe_gc_impl! {
    target => TypeId<'a, T>,
    params => ['a, T: StaticReflect],
    bounds => {
        Trace => always,
        TraceImmutable => always,
        GcSafe => always,
        GcRebrand => { where 'a: 'new_gc, T: 'new_gc },
        GcErase => { where 'a: 'min, T: 'min }
    },
    null_trace => always,
    branded_type => Self,
    erased_type => Self,
    NEEDS_TRACE => false,
    NEEDS_DROP => ::std::mem::needs_drop::<Self>(),
    visit => |self, visitor| { Ok(()) /* nop */ }
}
impl TypeId<'static> {
    /// Get the erased TypeId of the specified type `T`
    #[inline]
    pub const fn erased<T: StaticReflect>() -> TypeId<'static> {
        TypeId::<T>::get().erase()
    }
}
impl<T: StaticReflect> TypeId<'static, T> {
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
    pub const fn from_static(s: &'static TypeInfo<'static>) -> Self {
        TypeId {
            value: s,
            marker: PhantomData
        }
    }
}
impl<'tp, T: StaticReflect> TypeId<'tp, T> {
    /// Erase this type id,
    /// ignoring its statically-known generic
    /// parameters
    ///
    /// The generic parameters are unchecked, but are
    /// very useful for ensuring safety.
    #[inline]
    pub const fn erase(self) -> TypeId<'tp> {
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
    pub const fn type_ref(self) -> &'tp TypeInfo<'tp> {
        self.value
    }
    /// Create a [TypeId] from the specified reference
    #[inline]
    pub const fn from_ref(tp: &'tp TypeInfo<'tp>) -> Self {
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
impl<'a, T: StaticReflect> TypeId<'a, *mut T> {
    /// The target of the pointer type
    ///
    /// NOTE: This relies on static typing information
    #[inline]
    pub const fn pointer_target(self) -> TypeId<'a, T> {
        TypeId::get()
    }
}
impl<'tp> From<&'tp TypeInfo<'tp>> for TypeId<'tp> {
    #[inline]
    fn from(static_type: &'tp TypeInfo<'tp>) -> Self {
        TypeId::from_ref(static_type)
    }
}
impl<T: StaticReflect> Display for TypeId<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}
impl<T: StaticReflect> Debug for TypeId<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypeId")
            .field(self.value)
            .finish()
    }
}

/// A indexed identifier of a field
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FieldId<'a> {
    /// The owner of the field
    pub owner: TypeId<'a>,
    /// The index of the field
    pub index: usize,
}

