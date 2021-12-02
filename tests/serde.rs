use static_reflect_derive::StaticReflect;
use static_reflect::StaticReflect;
use static_reflect::builtins::AsmStr;

use serde_test::Token;

#[derive(StaticReflect)]
#[repr(C)]
struct TargetStructure {
    i: i32,
    s: AsmStr
}

#[test]
fn structure() {
    serde_test::assert_ser_tokens(
        &TargetStructure::TYPE_INFO,
        &[
            Token::NewtypeVariant {
                name: "TypeInfo",
                variant: "structure"
            },
            Token::Struct { name: "StructureDef", len: 4},
            Token::Str("name"),
            Token::Str("TargetStructure"),
            Token::Str("fields"),
            Token::Seq { len: Some(2) },
            Token::Struct { name: "FieldDef", len: 4 },
            Token::Str("name"),
            Token::Some,
            Token::Str("i"),
            Token::Str("value_type"),
            Token::NewtypeVariant {
                name: "TypeInfo",
                variant: "integer",
            },
            Token::Struct { name: "IntType", len: 2 },
            Token::Str("size"),
            Token::UnitVariant { name: "IntSize", variant: "Int" },
            Token::Str("signed"),
            Token::Bool(true),
            Token::StructEnd,
            Token::Str("offset"),
            Token::U64(0),
            Token::Str("index"),
            Token::U64(0),
            Token::StructEnd,
            Token::Struct { name: "FieldDef", len: 4 },
            Token::Str("name"),
            Token::Some,
            Token::Str("s"),
            Token::Str("value_type"),
            Token::UnitVariant {
                name: "TypeInfo",
                variant: "str",
            },
            Token::Str("offset"),
            Token::U64(8), // needs to be word aligned..
            Token::Str("index"),
            Token::U64(1),
            Token::StructEnd,
            Token::SeqEnd,
            Token::Str("size"),
            Token::U64(24),
            Token::Str("alignment"),
            Token::U64(8),
            Token::StructEnd
        ]
    );
}
