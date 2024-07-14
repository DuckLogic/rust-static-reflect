use static_reflect::types::{TypeId, TypeInfo, UnionFieldDef, UntaggedUnionDef};
use static_reflect::{FieldReflect, StaticReflect};
use static_reflect_derive::StaticReflect;

use std::mem::{align_of, size_of};

#[derive(Copy, Clone, Debug, PartialEq, StaticReflect)]
#[repr(C)]
pub struct Nested {
    float: f64,
    number: u64,
}

#[repr(C)]
#[derive(StaticReflect)]
union SimpleUnion {
    pub text: *mut String,
    b: bool,
    f: f32,
    nested: Nested,
}

#[test]
fn test_union_types() {
    const EXPECTED_UNION: TypeInfo = TypeInfo::UntaggedUnion(&UntaggedUnionDef {
        name: "SimpleUnion",
        fields: &[
            SimpleUnion::NAMED_FIELD_INFO.text.erase(),
            SimpleUnion::NAMED_FIELD_INFO.b.erase(),
            SimpleUnion::NAMED_FIELD_INFO.f.erase(),
            SimpleUnion::NAMED_FIELD_INFO.nested.erase(),
        ],
        size: size_of::<SimpleUnion>(),
        alignment: align_of::<SimpleUnion>(),
    });
    assert_eq!(EXPECTED_UNION, SimpleUnion::TYPE_INFO);
    assert_eq!(
        SimpleUnion::NAMED_FIELD_INFO.text,
        UnionFieldDef {
            name: "text",
            value_type: TypeId::<*mut String>::get(),
            index: 0
        }
    );
    assert_eq!(
        SimpleUnion::NAMED_FIELD_INFO.b,
        UnionFieldDef {
            name: "b",
            value_type: TypeId::<bool>::get(),
            index: 1
        }
    );
    assert_eq!(
        SimpleUnion::NAMED_FIELD_INFO.f,
        UnionFieldDef {
            name: "f",
            value_type: TypeId::<f32>::get(),
            index: 2
        }
    );
    assert_eq!(
        SimpleUnion::NAMED_FIELD_INFO.nested,
        UnionFieldDef {
            name: "nested",
            value_type: TypeId::<Nested>::get(),
            index: 3
        }
    );
}
