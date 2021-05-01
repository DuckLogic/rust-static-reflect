//! Reflection information on function declarations
use crate::types::{TypeInfo};
use std::marker::PhantomData;

/// The declaration of a function whose information
/// is known to the static reflection system
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionDeclaration<'a, R = (), Args = ()> {
    /// The name of the function, as declared in the
    /// source code.
    pub name: &'a str,
    /// If the function is unsafe
    pub is_unsafe: bool,
    /// The location of the function (if known)
    ///
    /// Not all supported functions have a known location.
    pub location: Option<FunctionLocation>,
    /// The signature of the function, including
    /// its arguments and return types
    ///
    /// Unlike the [PhantomData], this is actually retained at runtime.
    pub signature: SignatureDef<'a>,
    /// PhantomData: The return type of the function 
    pub return_type: PhantomData<fn() -> R>,
    /// PhantomData: The argument types of the function
    pub arg_types: PhantomData<fn(Args) -> ()>,
}
impl<'a, R, Args> FunctionDeclaration<'a, R, Args> {
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
    pub fn erase(&'a self) -> &'a FunctionDeclaration<(), ()> {
        unsafe { &*(self as *const Self as *const FunctionDeclaration<(), ()>) }
    }
}
/// The definition of a function's signature
///
/// Includes its argument types, return type, and calling convention.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SignatureDef<'tp> {
    /// A list of argument types to the function
    pub argument_types: &'tp [TypeInfo<'tp>],
    /// The return type of the function
    pub return_type: &'tp TypeInfo<'tp>,
    /// The calling convention
    pub calling_convention: CallingConvention
}


/// The convention used to call code.
///
/// Currently, only the C calling convention is supported
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FunctionLocation {
    /// The function is in a dynamically linked library,
    /// which will need to be resolved by the linker
    DynamicallyLinked {
        /// The name to be linked against,
        /// or `None` if it's the same as the function's name
        link_name: Option<&'static str>
    },
    /// The function is referred to by an absolute (hardcoded) address
    AbsoluteAddress(*const ()),
}
