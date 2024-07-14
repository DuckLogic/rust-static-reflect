#![feature(
    never_type,
)]
use pretty_assertions::assert_eq;

use static_reflect::{field_offset, StaticReflect, FieldReflect};
use static_reflect::types::{TypeInfo, FieldDef, StructureDef, TypeId, CStyleEnumVariant, CStyleEnumDef, DiscriminantValue, IntType};
use std::mem::{size_of, align_of};

use static_reflect_derive::{StaticReflect};

#[derive(Copy, Clone, Debug, PartialEq, StaticReflect)]
#[repr(C)]
pub struct Nested {
    cycle: *mut SimpleStruct,
    float: f64,
    number: u64,
}

#[derive(StaticReflect)]
#[repr(C)]
pub struct SimpleStruct {
    // We can have pointers to anything
    text: *mut String,
    number: u32,
    float: f64,
    b: bool,
    unit: (),
    nested_struct: Nested
}

#[test]
fn test_struct_types() {
    const NESTED_TYPE: TypeInfo = TypeInfo::Structure(&const { StructureDef {
        name: "Nested",
        fields: &[
            Nested::NAMED_FIELD_INFO.cycle.erase(),
            Nested::NAMED_FIELD_INFO.float.erase(),
            Nested::NAMED_FIELD_INFO.number.erase(),
        ],
        size: size_of::<Nested>(),
        alignment: align_of::<Nested>()
    } });
    assert_eq!(Nested::TYPE_INFO, NESTED_TYPE);
    assert_eq!(Nested::NAMED_FIELD_INFO.cycle, FieldDef {
        name: Some("cycle"),
        value_type: TypeId::<*mut SimpleStruct>::get(),
        offset: field_offset!(Nested, cycle),
        index: 0
    });
    assert_eq!(Nested::NAMED_FIELD_INFO.float, FieldDef {
        name: Some("float"),
        value_type: TypeId::<f64>::get(),
        offset: field_offset!(Nested, float),
        index: 1
    });
    assert_eq!(Nested::NAMED_FIELD_INFO.number, FieldDef {
        name: Some("number"),
        value_type: TypeId::<u64>::get(),
        offset: field_offset!(Nested, number),
        index: 2
    });
    const FIELDS: &[FieldDef] = &[
        FieldDef {
            name: Some("text"),
            value_type: TypeId::erased::<*mut String>(),
            offset: field_offset!(SimpleStruct, text),
            index: 0
        },
        FieldDef {
            name: Some("number"),
            value_type: TypeId::erased::<u32>(),
            offset: field_offset!(SimpleStruct, number),
            index: 1
        },
        FieldDef {
            name: Some("float"),
            value_type: TypeId::erased::<f64>(),
            offset: field_offset!(SimpleStruct, float),
            index: 2
        },
        FieldDef {
            name: Some("b"),
            value_type: TypeId::erased::<bool>(),
            offset: field_offset!(SimpleStruct, b),
            index: 3
        },
        FieldDef {
            name: Some("unit"),
            value_type: TypeId::erased::<()>(),
            offset: field_offset!(SimpleStruct, unit),
            index: 4
        },
        FieldDef {
            name: Some("nested_struct"),
            // NOTE: We already checked Nested::STATIC_TYPE
            value_type: TypeId::erased::<Nested>(),
            offset: field_offset!(SimpleStruct, nested_struct),
            index: 5
        },
    ];
    assert_eq!(
        SimpleStruct::TYPE_INFO,
        TypeInfo::Structure(&const { StructureDef {
            name: "SimpleStruct",
            fields: FIELDS,
            size: size_of::<SimpleStruct>(),
            alignment: align_of::<SimpleStruct>(),
        } })
    );
}


#[derive(StaticReflect)]
#[repr(C)]
struct SimpleTupleStruct(*mut String, f32, Nested);

#[test]
fn test_tuple_struct() {
    const FIELDS: &[FieldDef] = &[
        FieldDef {
            name: None,
            value_type: TypeId::erased::<*mut String>(),
            offset: field_offset!(SimpleTupleStruct, 0),
            index: 0
        },
        FieldDef {
            name: None,
            value_type: TypeId::erased::<f32>(),
            offset: field_offset!(SimpleTupleStruct, 1),
            index: 1
        },
        FieldDef {
            name: None,
            // NOTE: We already checked Nested::STATIC_TYPE
            value_type: TypeId::erased::<Nested>(),
            offset: field_offset!(SimpleTupleStruct, 2),
            index: 2
        },
    ];
    assert_eq!(SimpleTupleStruct::NAMED_FIELD_INFO.0.erase(), FIELDS[0]);
    assert_eq!(SimpleTupleStruct::NAMED_FIELD_INFO.1.erase(), FIELDS[1]);
    assert_eq!(SimpleTupleStruct::NAMED_FIELD_INFO.2.erase(), FIELDS[2]);
    assert_eq!(
        SimpleTupleStruct::TYPE_INFO,
        TypeInfo::Structure(&StructureDef {
            name: "SimpleTupleStruct",
            fields: FIELDS,
            size: size_of::<SimpleTupleStruct>(),
            alignment: align_of::<SimpleTupleStruct>(),
        })
    );
}

#[derive(StaticReflect)]
#[repr(C)]
#[allow(dead_code)]
enum SimpleEnum {
    Zero,
    Two = 2,
    Eight = 8,
    Four = 4,
    Implicit
}

#[test] // Currently fails because of #2
fn test_simple_enum(){
    assert_eq!(
        SimpleEnum::TYPE_INFO,
        TypeInfo::CStyleEnum(&CStyleEnumDef {
            name: "SimpleEnum",
            discriminant: IntType::ISIZE,
            variants: &[
                CStyleEnumVariant {
                    index: 0,
                    name: "Zero",
                    discriminant: DiscriminantValue::Default {
                        declaration_index: 0,
                    },
                },
                CStyleEnumVariant {
                    index: 1,
                    name: "Two",
                    discriminant: DiscriminantValue::ExplicitInteger {
                        bits: 2,
                    },
                },
                CStyleEnumVariant {
                    index: 2,
                    name: "Eight",
                    discriminant: DiscriminantValue::ExplicitInteger {
                        bits: 8,
                    },
                },
                CStyleEnumVariant {
                    index: 3,
                    name: "Four",
                    discriminant: DiscriminantValue::ExplicitInteger {
                        bits: 4,
                    },
                },
                CStyleEnumVariant {
                    index: 4,
                    name: "Implciit",
                    discriminant: DiscriminantValue::ImplicitlyOffset {
                        bits: 5,
                    },
                }
            ]
        })
    )
}

#[derive(StaticReflect)]
#[repr(C)]
struct OpaqueArray {
    #[reflect(assume_repr = "i8")]
    first: u8,
    #[static_reflect(opaque_array)]
    array: [*mut String; 42]
}

#[test]
fn test_options() {
    const OPAQUE_ARRAY_TYPE: TypeInfo = TypeInfo::Structure(&StructureDef {
        name: "OpaqueArray",
        fields: &[
            OpaqueArray::NAMED_FIELD_INFO.first.erase(),
            OpaqueArray::NAMED_FIELD_INFO.array.erase(),
        ],
        size: size_of::<OpaqueArray>(),
        alignment: align_of::<OpaqueArray>()
    });
    assert_eq!(OPAQUE_ARRAY_TYPE, OpaqueArray::TYPE_INFO);
    assert_eq!(OpaqueArray::NAMED_FIELD_INFO.first, FieldDef {
        name: Some("first"),
        value_type: TypeId::<i8>::get(), // It's actually a 'u8', but we assume_repr
        offset: field_offset!(OpaqueArray, first),
        index: 0
    });
    assert_eq!(OpaqueArray::NAMED_FIELD_INFO.array, FieldDef {
        name: Some("array"),
        value_type: TypeId::<*mut String>::get(),
        offset: field_offset!(OpaqueArray, array),
        index: 1
    });
}
