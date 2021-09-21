//! Reflection information on function declarations
use educe::Educe;
use crate::types::{TypeInfo};
use std::marker::PhantomData;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "serde")]
use zerogc_derive::GcDeserialize;

use zerogc::{CollectorId};
use zerogc::array::{GcArray, GcString};
use zerogc::epsilon::{EpsilonCollectorId};
use zerogc_derive::{Trace, NullTrace};


/// The declaration of a function whose information
/// is known to the static reflection system
#[derive(Trace, Educe)]
#[cfg_attr(feature = "serde", derive(GcDeserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", bound(serialize = "")))]
#[educe(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[zerogc(copy, collector_ids(Id), ignore_params(R, Args), serde(require_simple_alloc))]
pub struct FunctionDeclaration<'gc, R: 'gc = (), Args: 'gc = (), Id: CollectorId = EpsilonCollectorId> {
    /// The name of the function, as declared in the
    /// source code.
    pub name: GcString<'gc, Id>,
    /// If the function is unsafe
    pub is_unsafe: bool,
    /// The location of the function (if known)
    ///
    /// Not all supported functions have a known location.
    pub location: Option<FunctionLocation<'gc, Id>>,
    /// The signature of the function, including
    /// its arguments and return types
    ///
    /// Unlike the [PhantomData], this is actually retained at runtime.
    pub signature: SignatureDef<'gc, Id>,
    /// PhantomData: The return type of the function 
    pub return_type: PhantomData<fn() -> R>,
    /// PhantomData: The argument types of the function
    pub arg_types: PhantomData<fn(Args) -> ()>,
}
impl<'gc, R, Args, Id: CollectorId> FunctionDeclaration<'gc, R, Args, Id> {
    /// If the function has a known location at runtime
    ///
    /// If this is false, it wont actually be possible
    /// to call the function later. If it is true,
    /// then you can.
    #[inline]
    pub fn has_known_location(&self) -> bool {
        self.location.is_some()
    }
    /// Erase all statically known type information
    #[inline]
    pub const fn erase(&self) -> &'_ FunctionDeclaration<'gc, (), (), Id> {
        unsafe { &*(self as *const Self as *const FunctionDeclaration<(), (), Id>) }
    }
}
/// The definition of a function's signature
///
/// Includes its argument types, return type, and calling convention.
#[derive(Educe, Trace)]
#[cfg_attr(feature = "serde", derive(Serialize, GcDeserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", bound(serialize = "")))]
#[educe(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[zerogc(copy, collector_ids(Id), serde(require_simple_alloc))]
pub struct SignatureDef<'gc, Id: CollectorId = EpsilonCollectorId> {
    /// A list of argument types to the function
    pub argument_types: GcArray<'gc, TypeInfo<'gc, Id>, Id>,
    /// The return type of the function
    pub return_type: TypeInfo<'gc, Id>,
    /// The calling convention
    #[cfg_attr(feature = "serde", zerogc(serde(delegate)))]
    pub calling_convention: CallingConvention
}


/// The convention used to call code.
///
/// Currently, only the C calling convention is supported
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, NullTrace)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CallingConvention {
    /// Matches the target's C calling convention `extern "C"`
    ///
    /// This is the default calling convention.
    /// It has a stable ABI and is used to call external functions
    StandardC,
}
impl Default for CallingConvention {
    #[inline]
    fn default() -> Self {
        CallingConvention::StandardC
    }
}

/// The location of the function
///
/// Gives specific information on which function to invoke
#[derive(Educe, Trace)]
#[cfg_attr(feature = "serde", derive(GcDeserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case", bound(serialize = "")))]
#[educe(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[zerogc(copy, collector_ids(Id), serde(require_simple_alloc))]
pub enum FunctionLocation<'gc, Id: CollectorId = EpsilonCollectorId> {
    /// The function is in a dynamically linked library,
    /// which will need to be resolved by the linker
    DynamicallyLinked {
        /// The name to be linked against,
        /// or `None` if it's the same as the function's name
        link_name: Option<GcString<'gc, Id>>
    },
    /// The function is referred to by an absolute (hardcoded) address
    AbsoluteAddress {
        /// The actual addresss
        ///
        /// WARNING: It is posssible for entirely safe code to construct
        /// aribtrary pointer values here. The value here should not (necessarily) be
        /// trusted for memory safety.
        #[zerogc(unsafe_skip_trace)]
        #[cfg_attr(feature = "serde", serde(serialize_with = "serialize::serialize_addr"))]
        #[cfg_attr(feature = "serde", zerogc(serde(deserialize_with = "serialize::deserialize_addr")))]
        addr: *const ()
    },
}

#[cfg(feature = "serde")]
mod serialize {
    use serde::{Serialize, Serializer, Deserialize, Deserializer};
    pub fn serialize_addr<T, S: Serializer>(ptr: &*const T, ser: S) -> Result<S::Ok, S::Error> {
        usize::serialize(&(*ptr as usize), ser)
    }
    pub fn deserialize_addr<'de, T, D: Deserializer<'de>>(de: D) -> Result<*const T, D::Error> {
        Ok(usize::deserialize(de)? as *const T)
    }
}
