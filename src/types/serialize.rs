use std::marker::PhantomData;
use std::fmt::{self, Formatter};

use serde::de::{self, Deserializer, EnumAccess, DeserializeSeed, Visitor, Error as DeserializeError};

use super::{TypeInfo, TypeAllocContext};

pub trait DeserializeAlloc<'a, 'de> {
    
}

pub struct AllocDeserializer<'a, 'de, A: TypeAllocContext, S: DeserializeSeed<'de>> {
    pub context: &'a A,
    pub marker: PhantomData<fn(&'de ()) -> T>
}

impl<'a, 'de, A: TypeAllocContext, T: Deserialize<'de>> DeserializeSeed<'de> for DeserializeAlloc<'a, A> {
    type Value = T;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> 
        where D: Deserializer<'de> {
        let visitor = TypeInfoVisitor {
            context: self.context,
            marker: PhantomData
        };
        if deserializer.is_human_readable() {
            deserialize.deserialize_any(visitor)
        } else {
            deserialize.deserialize_enum("TypeInfo", todo!()e, visitor)
        }
    }
}