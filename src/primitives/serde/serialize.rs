use log::trace;
use serde::{ser, Serialize};
use std::collections::BTreeMap;

use crate::{primitives::{blob::VicBlob, Primitives}, topics::{TopicKey, TopicKeyHandle, TopicKeyProvider}};

// Define a custom error type for serialization errors
#[derive(Debug)]
pub enum PrimitiveError {
    Message(String),
}

impl ser::Error for PrimitiveError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        PrimitiveError::Message(msg.to_string())
    }
}

impl std::fmt::Display for PrimitiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for PrimitiveError {}

pub type PrimitiveResult<T> = Result<T, PrimitiveError>;

pub struct PrimitiveSerializer {
    pub prefix: TopicKeyHandle,
    pub map: BTreeMap<TopicKeyHandle, Primitives>,
}

pub fn to_map<T>(value: &T) -> PrimitiveResult<BTreeMap<TopicKeyHandle, Primitives>>
where
    T: Serialize,
{
    let mut serializer = PrimitiveSerializer {
        prefix: TopicKey::from_str("").handle(),
        map: BTreeMap::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.map)
}

impl<'a> ser::Serializer for &'a mut PrimitiveSerializer {
    type Ok = ();
    type Error = PrimitiveError;

    type SerializeSeq = SerializeSeq<'a>;
    type SerializeTuple = SerializeSeq<'a>;
    type SerializeTupleStruct = SerializeSeq<'a>;
    type SerializeTupleVariant = SerializeSeq<'a>;
    type SerializeMap = SerializeMap<'a>;
    type SerializeStruct = SerializeStruct<'a>;
    type SerializeStructVariant = SerializeStructVariant<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let key = self.prefix.clone();
        self.map.insert(key, Primitives::Boolean(v));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let key = self.prefix.clone();
        self.map.insert(key, Primitives::Integer(v));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if v <= i64::MAX as u64 {
            self.serialize_i64(v as i64)
        } else {
            Err(PrimitiveError::Message("u64 value too large".into()))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let key = self.prefix.clone();
        self.map.insert(key, Primitives::Float(v));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let key = self.prefix.clone();
        self.map.insert(key, Primitives::Text(v.to_string()));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let key = self.prefix.clone();
        self.map
            .insert(key, Primitives::Blob(VicBlob::new_from_data(v.to_vec())));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // Do nothing for None values
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // Do nothing for unit types
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        // Do nothing for unit structs
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let key = self.prefix.clone();
        self.map.insert(key, Primitives::Text(variant.to_string()));
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let current_prefix = self.prefix.clone();

        let new_prefix = self.prefix.display_name() + "/" + variant;
        let new_prefix = TopicKey::from_str(new_prefix.as_str());
        self.prefix = new_prefix.handle();
        value.serialize(&mut *self)?;
        self.prefix = current_prefix;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq {
            ser: self,
            index: 0,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let new_prefix= self.prefix.add_suffix(TopicKey::from_str(variant));
        self.prefix = new_prefix.handle();
        Ok(SerializeSeq {
            ser: self,
            index: 0,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            ser: self,
            key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // Create a new StructType and serialize it
        let new_prefix = self.prefix.add_suffix(TopicKey::from_str("_type"));
        self.map.insert(
            new_prefix.handle(),
            Primitives::StructType(name.to_string()),
        );

        Ok(SerializeStruct { ser: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let current_prefix = self.prefix.clone();

        let new_prefix = self.prefix.add_suffix(TopicKey::from_str(variant));
        self.prefix = new_prefix.handle();
        
        Ok(SerializeStructVariant {
            ser: self,
            original_prefix: current_prefix,
        })
    }
}

// Helper struct for serializing sequences (arrays)
pub struct SerializeSeq<'a> {
    ser: &'a mut PrimitiveSerializer,
    index: usize,
}

impl<'a> ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = ();
    type Error = PrimitiveError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        let current_prefix = self.ser.prefix.clone();
        let new_prefix = self.ser.prefix.add_suffix(TopicKey::from_str(&self.index.to_string()));
        self.ser.prefix = new_prefix.handle();
        self.index += 1;
        value.serialize(&mut *self.ser)?;
        self.ser.prefix = current_prefix;
        Ok(())
    }

    fn end(self) -> Result<(), PrimitiveError> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for SerializeSeq<'a> {
    type Ok = ();
    type Error = PrimitiveError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), PrimitiveError> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for SerializeSeq<'a> {
    type Ok = ();
    type Error = PrimitiveError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), PrimitiveError> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for SerializeSeq<'a> {
    type Ok = ();
    type Error = PrimitiveError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<(), PrimitiveError> {
        let prefix = self.ser.prefix.clone();
        let mut new_prefix = prefix.key().clone();
        if !new_prefix.sections.is_empty() {
            new_prefix.sections.pop();
        }
        self.ser.prefix = new_prefix.handle();
        Ok(())
    }
}

// Helper struct for serializing maps
pub struct SerializeMap<'a> {
    ser: &'a mut PrimitiveSerializer,
    key: Option<TopicKeyHandle>,
}

impl<'a> ser::SerializeMap for SerializeMap<'a> {
    type Ok = ();
    type Error = PrimitiveError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        let mut key_serializer = KeySerializer::default();
        key.serialize(&mut key_serializer)?;
        let key = self.ser.prefix.add_suffix(key_serializer.key.key().clone());
        self.key = Some(key.handle());
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        let current_prefix = self.ser.prefix.clone();
        self.ser.prefix = self.key.clone().unwrap();
        value.serialize(&mut *self.ser)?;
        self.ser.prefix = current_prefix;
        self.key = None;
        Ok(())
    }

    fn end(self) -> Result<(), PrimitiveError> {
        Ok(())
    }
}

// Helper struct for serializing structs
pub struct SerializeStruct<'a> {
    ser: &'a mut PrimitiveSerializer,
}

impl<'a> ser::SerializeStruct for SerializeStruct<'a> {
    type Ok = ();
    type Error = PrimitiveError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        let current_prefix = self.ser.prefix.clone();
        let new_prefix = self.ser.prefix.add_suffix(TopicKey::from_str(key));
        self.ser.prefix = new_prefix.handle();
        value.serialize(&mut *self.ser)?;
        self.ser.prefix = current_prefix;

        Ok(())
    }

    fn end(self) -> Result<(), PrimitiveError> {
        Ok(())
    }
}

// Helper struct for serializing struct variants
pub struct SerializeStructVariant<'a> {
    ser: &'a mut PrimitiveSerializer,
    original_prefix: TopicKeyHandle,
}

impl<'a> ser::SerializeStructVariant for SerializeStructVariant<'a> {
    type Ok = ();
    type Error = PrimitiveError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), PrimitiveError>
    where
        T: Serialize,
    {
        let current_prefix = self.ser.prefix.clone();

        let new_prefix = self.ser.prefix.add_suffix(TopicKey::from_str(key));
        self.ser.prefix = new_prefix.handle();
        value.serialize(&mut *self.ser)?;
        self.ser.prefix = current_prefix;
        Ok(())
    }

    fn end(self) -> Result<(), PrimitiveError> {
        self.ser.prefix = self.original_prefix.clone();
        Ok(())
    }
}

// Serializer for map keys

pub struct KeySerializer {
    key: TopicKeyHandle,
}

impl Default for KeySerializer {
    fn default() -> Self {
        KeySerializer {
            key: TopicKey::from_str("").handle(),
        }
    }
}

impl<'a> ser::Serializer for &'a mut KeySerializer {
    type Ok = ();
    type Error = PrimitiveError;

    type SerializeSeq = Impossible<(), PrimitiveError>;
    type SerializeTuple = Impossible<(), PrimitiveError>;
    type SerializeTupleStruct = Impossible<(), PrimitiveError>;
    type SerializeTupleVariant = Impossible<(), PrimitiveError>;
    type SerializeMap = Impossible<(), PrimitiveError>;
    type SerializeStruct = Impossible<(), PrimitiveError>;
    type SerializeStructVariant = Impossible<(), PrimitiveError>;

    fn serialize_str(self, value: &str) -> Result<(), PrimitiveError> {
        self.key = TopicKey::from_str(value).handle();
        Ok(())
    }

    fn serialize_unit(self) -> Result<(), PrimitiveError> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_bool(self, _v: bool) -> Result<(), PrimitiveError> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_i8(self, _v: i8) -> Result<(), PrimitiveError> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(PrimitiveError::Message("Unsupported key type".into()))
    }

    // Implement similar methods for other unsupported types
    // ...
}

// A helper struct that represents impossible serialization cases
pub struct Impossible<T, E> {
    _marker: std::marker::PhantomData<(T, E)>,
}

// Implementing serialization traits for Impossible
impl<T, E> ser::SerializeSeq for Impossible<T, E> {
    type Ok = T;
    type Error = PrimitiveError;
    fn serialize_element<S: ?Sized>(&mut self, _value: &S) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Not supported".into()))
    }
}

impl<T, E> ser::SerializeTuple for Impossible<T, E> {
    type Ok = T;
    type Error = PrimitiveError;
    fn serialize_element<S: ?Sized>(&mut self, _value: &S) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Not supported".into()))
    }
}

impl<T, E> ser::SerializeTupleStruct for Impossible<T, E> {
    type Ok = T;
    type Error = PrimitiveError;
    fn serialize_field<S: ?Sized>(&mut self, _value: &S) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Not supported".into()))
    }
}

impl<T, E> ser::SerializeTupleVariant for Impossible<T, E> {
    type Ok = T;
    type Error = PrimitiveError;
    fn serialize_field<S: ?Sized>(&mut self, _value: &S) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Not supported".into()))
    }
}

impl<T, E> ser::SerializeMap for Impossible<T, E> {
    type Ok = T;
    type Error = PrimitiveError;
    fn serialize_key<S: ?Sized>(&mut self, _key: &S) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn serialize_value<S: ?Sized>(&mut self, _value: &S) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Not supported".into()))
    }
}

impl<T, E> ser::SerializeStruct for Impossible<T, E> {
    type Ok = T;
    type Error = PrimitiveError;
    fn serialize_field<S: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &S,
    ) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Not supported".into()))
    }
}

impl<T, E> ser::SerializeStructVariant for Impossible<T, E> {
    type Ok = T;
    type Error = PrimitiveError;
    fn serialize_field<S: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &S,
    ) -> Result<(), PrimitiveError>
    where
        S: Serialize,
    {
        Err(PrimitiveError::Message("Not supported".into()))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Err(PrimitiveError::Message("Not supported".into()))
    }
}

// Testing the serializer
#[cfg(test)]
mod tests_serde {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]

    struct TestSimpleStruct {
        simple_int: i32,
        simple_str: String,
    }

    impl Default for TestSimpleStruct {
        fn default() -> Self {
            TestSimpleStruct {
                simple_int: 7,
                simple_str: String::from("test_string"),
            }
        }
    }

    #[derive(Serialize)]

    struct ComplexStruct {
        a: i32,
        b: TestSimpleStruct,
        c: Vec<i32>,
        d: BTreeMap<String, TestSimpleStruct>,
    }

    impl Default for ComplexStruct {
        fn default() -> Self {
            let mut map = BTreeMap::new();
            map.insert("test_1".to_string(), TestSimpleStruct::default());
            map.insert("test_2".to_string(), TestSimpleStruct::default());
            ComplexStruct {
                a: 7,
                b: TestSimpleStruct::default(),
                c: vec![1, 2, 3],
                d: map,
            }
        }
    }

    #[test]
    fn test_simple_struct() {
        sensible_env_logger::safe_init!();
        let test_struct = TestSimpleStruct::default();
        let result = to_map(&test_struct);
        println!("{:?}", result);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.len(), 3);

    }

    #[test]
    fn test_complex_struct() {
        let test_struct = ComplexStruct::default();
        let result = to_map(&test_struct);
        assert!(result.is_ok());
        let map = result.unwrap();
     
    }
}
