use std::alloc::Layout;

use derive_more::From;
use libc::{c_char, c_int, c_uchar, c_uint, c_void, intptr_t};
use serde::{Deserialize, Serialize};

pub trait LayoutOf {
    fn layout(&self) -> Layout;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Struct {
    #[serde(rename = "struct")]
    pub name: String,
    #[serde(default)]
    pub fields: Vec<(String, Type)>,
}

impl LayoutOf for Struct {
    fn layout(&self) -> Layout {
        let layouts = self.fields.iter().map(|(_, t)| t.layout()).collect::<Vec<_>>
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, From)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Type {
    Int(Int),
    Struct(Struct),
    Ptr(Ptr),
    Void(Void),
}

impl Type {
    pub fn void() -> Self {
        Type::Void(Void::Void)
    }
}

impl LayoutOf for Type {
    fn layout(&self) -> Layout {
        match self {
            Type::Int(int) => int.layout(),
            Type::Struct(r#struct) => r#struct.layout(),
            Type::Ptr(ptr) => ptr.layout(),
            Type::Void(void) => void.layout(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, From)]
#[serde(rename_all = "snake_case")]
pub enum Void {
    Void,
}

impl LayoutOf for Void {
    fn layout(&self) -> Layout {
        Layout::new::<c_void>()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Ptr {
    #[serde(rename = "ptr")]
    pub deref_target: Box<Type>,
}

impl Ptr {
    pub fn new<T: Into<Type>>(target: T) -> Self {
        Ptr {
            deref_target: Box::new(target.into()),
        }
    }
}

impl LayoutOf for Ptr {
    fn layout(&self) -> Layout {
        Layout::new::<intptr_t>()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Int {
    Char,
    UChar,
    Int,
    UInt,
    U8,
    I8,
}

impl LayoutOf for Int {
    fn layout(&self) -> Layout {
        match self {
            Int::Char => Layout::new::<c_char>(),
            Int::UChar => Layout::new::<c_uchar>(),
            Int::Int => Layout::new::<c_int>(),
            Int::UInt => Layout::new::<c_uint>(),
            Int::U8 => Layout::new::<u8>(),
            Int::I8 => Layout::new::<i8>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::c_types::{Int, Ptr, Struct, Type, Void};
    use color_eyre::eyre::eyre;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use std::fmt::Debug;

    fn check_parse<T>(json: Value, target: T)
    where
        T: Debug + PartialEq + for<'de> Deserialize<'de>,
    {
        assert_eq!(
            serde_json::from_value::<T>(json.clone())
                .map_err(|e| eyre!(e))
                .unwrap(),
            target
        );
    }

    fn check_repr<T>(value: T, target: Value)
    where
        T: Serialize,
    {
        assert_eq!(
            serde_json::to_value(value).map_err(|e| eyre!(e)).unwrap(),
            target
        );
    }

    #[test]
    fn check_direct_int_parsing() {
        check_parse(json!("char"), Int::Char);
        check_parse(json!("uchar"), Int::UChar);
        check_parse(json!("u8"), Int::U8);
    }

    #[test]
    fn check_void_parsing() {
        check_parse(json!("void"), Type::void());
        check_parse(json!("void"), Void::Void);
    }

    #[test]
    fn check_void_repr() {
        check_repr(Type::void(), json!("void"));
        check_repr(Void::Void, json!("void"));
    }

    #[test]
    fn check_int_parsing() {
        check_parse::<Type>(json!("char"), Int::Char.into());
    }

    #[test]
    fn check_ptr_parsing() {
        check_parse::<Ptr>(json!({"ptr": "char"}), Ptr::new(Int::Char));
        check_parse::<Type>(json!({"ptr": "char"}), Ptr::new(Int::Char).into());
        check_parse::<Type>(
            json!({"ptr": {"struct": "my_cool_struct"}}),
            Ptr::new(Struct {
                name: "my_cool_struct".into(),
                fields: vec![],
            })
            .into(),
        );
    }

    #[test]
    fn check_struct_parsing() {
        check_parse::<Struct>(
            json!({
                "struct": "my_cool_struct",
                "fields": [
                    ["a", "int"],
                    ["b", "uint"],
                    ["c", {"ptr" : "u8"}]
                ]
            }),
            Struct {
                name: "my_cool_struct".into(),
                fields: vec![
                    ("a".into(), Int::Int.into()),
                    ("b".into(), Int::UInt.into()),
                    ("c".into(), Ptr::new(Int::U8).into()),
                ],
            },
        )
    }
}
