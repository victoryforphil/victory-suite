use serde::de::{self, Deserialize, Deserializer, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use std::collections::BTreeMap;

use crate::{primitives::Primitives, topics::TopicKey};

pub struct PrimitiveDeserializer<'de> {
    flat_map: &'de BTreeMap<String, Primitives>,
    path: String,
}

impl<'de> PrimitiveDeserializer<'de> {
    pub fn new(flat_map: &'de BTreeMap<String, Primitives>) -> Self {
        PrimitiveDeserializer {
            flat_map,
            path: String::new(),
        }
    }

    fn get_value(&self) -> Option<&Primitives> {
        self.flat_map.get(&self.path)
    }

    fn enter(&mut self, key: &str) {
        self.path.push_str(key);
        self.path.push('/');
    }

    fn exit(&mut self) {
        // Struct/field/a/ -> Struct/field/

        // Delete the last key
        let mut topic = TopicKey::from_str(&self.path);
        // Remove the last key
        topic.sections.pop();
        self.path = topic.display_name() + "/";
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut PrimitiveDeserializer<'de> {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(primitive) = self.get_value() {
            match primitive {
                Primitives::Boolean(b) => visitor.visit_bool(*b),
                Primitives::Integer(i) => visitor.visit_i64(*i),
                Primitives::Float(f) => visitor.visit_f64(*f),
                Primitives::Text(s) => visitor.visit_str(s),
                Primitives::Blob(blob) => visitor.visit_bytes(blob.data.as_slice()),
                Primitives::List(_) => self.deserialize_seq(visitor),
                Primitives::Unset => visitor.visit_unit(),
                _ => Err(de::Error::custom("Unsupported primitive type")),
            }
        } else {
            self.deserialize_map(visitor)
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Add the struct name to the path
        self.enter(name);
        let res = visitor.visit_map(StructAccess {
            de: self,
            fields: fields.iter().cloned().collect(),
            field_index: 0,
        });
        self.exit();
        res
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let prefix = self.path.clone();
        let indices = self.collect_sequence_indices(&prefix);
        let seq_access = SeqAccessImpl {
            de: self,
            indices,
            index: 0,
        };
        visitor.visit_seq(seq_access)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let prefix = self.path.clone();
        let keys = self.collect_map_keys(&prefix);
        let map_access = MapAccessImpl {
            de: self,
            keys,
            key_index: 0,
        };
        visitor.visit_map(map_access)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Boolean(b)) = self.get_value() {
            visitor.visit_bool(*b)
        } else {
            Err(de::Error::custom("Expected boolean"))
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i64(*i)
        } else {
            Err(de::Error::custom("Expected integer"))
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Float(f)) = self.get_value() {
            visitor.visit_f64(*f)
        } else {
            Err(de::Error::custom("Expected float"))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Text(s)) = self.get_value() {
            visitor.visit_string(s.clone())
        } else {
            Err(de::Error::custom("Expected string"))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Blob(blob)) = self.get_value() {
            visitor.visit_bytes(blob.data.as_slice())
        } else {
            Err(de::Error::custom("Expected bytes"))
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i8(*i as i8)
        } else {
            Err(de::Error::custom("Expected i8"))
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i16(*i as i16)
        } else {
            Err(de::Error::custom("Expected i16"))
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_i32(*i as i32)
        } else {
            Err(de::Error::custom(format!(
                "Expected i32, got {:?}",
                self.get_value()
            )))
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u8(*i as u8)
        } else {
            Err(de::Error::custom("Expected u8"))
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u16(*i as u16)
        } else {
            Err(de::Error::custom("Expected u16"))
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u32(*i as u32)
        } else {
            Err(de::Error::custom("Expected u32"))
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Integer(i)) = self.get_value() {
            visitor.visit_u64(*i as u64)
        } else {
            Err(de::Error::custom("Expected u64"))
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Float(f)) = self.get_value() {
            visitor.visit_f32(*f as f32)
        } else {
            Err(de::Error::custom("Expected f32"))
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Text(s)) = self.get_value() {
            if let Some(c) = s.chars().next() {
                visitor.visit_char(c)
            } else {
                Err(de::Error::custom("Expected char"))
            }
        } else {
            Err(de::Error::custom("Expected char"))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Text(s)) = self.get_value() {
            visitor.visit_str(s)
        } else {
            Err(de::Error::custom("Expected string"))
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Blob(blob)) = self.get_value() {
            visitor.visit_byte_buf(blob.data.clone())
        } else {
            Err(de::Error::custom("Expected byte buffer"))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(primitive) = self.get_value() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Unset) = self.get_value() {
            visitor.visit_unit()
        } else {
            Err(de::Error::custom("Expected unit"))
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(Primitives::Unset) = self.get_value() {
            visitor.visit_unit()
        } else {
            Err(de::Error::custom("Expected unit"))
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Ok(visitor.visit_seq(SeqAccessImpl {
            de: self,
            indices: (0..len).collect(),
            index: 0,
        })?)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Ok(visitor.visit_seq(SeqAccessImpl {
            de: self,
            indices: (0..len).collect(),
            index: 0,
        })?)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(EnumAccessImpl { de: self })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.path)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
struct EnumAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
}

impl<'de, 'a> de::EnumAccess<'de> for EnumAccessImpl<'a, 'de> {
    type Error = de::value::Error;
    type Variant = VariantAccessImpl<'a, 'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // The enum variant name is expected to be at the current path
        if let Some(Primitives::Text(variant_name)) = self.de.get_value() {
            let val = seed.deserialize(variant_name.as_str().into_deserializer())?;
            Ok((val, VariantAccessImpl { de: self.de }))
        } else {
            Err(de::Error::custom("Expected enum variant as text"))
        }
    }
}

struct VariantAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
}

impl<'de, 'a> de::VariantAccess<'de> for VariantAccessImpl<'a, 'de> {
    type Error = de::value::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        // Enter the path to the variant's associated data
        self.de.enter(""); // Entering empty string to append '/' to path
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.enter(""); // Entering empty string to append '/' to path
        let indices = self.de.collect_sequence_indices(&self.de.path);
        let value = visitor.visit_seq(SeqAccessImpl {
            de: self.de,
            indices: indices,
            index: 0,
        })?;
        self.de.exit();
        Ok(value)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.enter(""); // Entering empty string to append '/' to path
        let value = visitor.visit_map(StructAccess {
            de: self.de,
            fields: _fields.iter().cloned().collect(),
            field_index: 0,
        })?;
        self.de.exit();
        Ok(value)
    }
}

impl<'de, 'a> PrimitiveDeserializer<'de> {
    fn collect_sequence_indices(&self, prefix: &str) -> Vec<usize> {
        let mut indices = Vec::new();
        let prefix_len = prefix.len();
        for key in self.flat_map.keys() {
            if key.starts_with(&format!("{}", prefix)) {
                let remainder = &key[prefix_len..];
                if let Some((key_part, _)) = remainder.split_once('/') {
                    if let Ok(idx) = key_part.parse::<usize>() {
                        indices.push(idx);
                    }
                }
            }
        }
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    fn collect_map_keys(&self, prefix: &str) -> Vec<String> {
        let mut keys = Vec::new();
        let prefix_len = prefix.len();
        for key in self.flat_map.keys() {
            if key.starts_with(&format!("{}", prefix)) {
                let remainder = &key[prefix_len..];
                if let Some((key_part, _)) = remainder.split_once('/') {
                    keys.push(key_part.to_string());
                } else {
                    keys.push(remainder.to_string());
                }
            }
        }
        keys.sort();
        keys.dedup();
        keys
    }
}

// Implement MapAccess for structs
struct StructAccess<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
    fields: Vec<&'static str>,
    field_index: usize,
}

impl<'de, 'a> MapAccess<'de> for StructAccess<'a, 'de> {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.field_index < self.fields.len() {
            let key = self.fields[self.field_index];
            self.field_index += 1;
            seed.deserialize(key.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let key = self.fields[self.field_index - 1];
        self.de.enter(key);
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value)
    }
}

// Implement SeqAccess for sequences
struct SeqAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
    indices: Vec<usize>,
    index: usize,
}

impl<'de, 'a> SeqAccess<'de> for SeqAccessImpl<'a, 'de> {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index < self.indices.len() {
            let idx = self.indices[self.index];
            self.index += 1;
            self.de.enter(&idx.to_string());
            let value = seed.deserialize(&mut *self.de)?;
            self.de.exit();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

// Implement MapAccess for maps
struct MapAccessImpl<'a, 'de> {
    de: &'a mut PrimitiveDeserializer<'de>,
    keys: Vec<String>,
    key_index: usize,
}

impl<'de, 'a> MapAccess<'de> for MapAccessImpl<'a, 'de> {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.key_index < self.keys.len() {
            let key = &self.keys[self.key_index];
            self.key_index += 1;
            seed.deserialize(key.as_str().into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let key = &self.keys[self.key_index - 1];
        self.de.enter(key);
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq, Deserialize)]
    struct TestSimpleStruct {
        simple_int: i32,
        simple_string: String,
    }
    #[derive(Debug, PartialEq, Deserialize)]
    struct TestComplexStruct {
        complex_int: i32,
        complex_string: String,
        simple_struct: TestSimpleStruct,
    }

    #[test]
    fn test_deserialize_primitives() {
        let mut flat_map = BTreeMap::new();
        flat_map.insert(
            "TestComplexStruct/".to_string(),
            Primitives::StructType("TestComplexStruct".to_string()),
        );
        flat_map.insert(
            "TestComplexStruct/complex_int/".to_string(),
            Primitives::Integer(42),
        );
        flat_map.insert(
            "TestComplexStruct/complex_string/".to_string(),
            Primitives::Text("Test String".to_string()),
        );
        flat_map.insert(
            "TestComplexStruct/simple_struct/TestSimpleStruct/".to_string(),
            Primitives::StructType("TestSimpleStruct".to_string()),
        );

        flat_map.insert(
            "TestComplexStruct/simple_struct/TestSimpleStruct/simple_int/".to_string(),
            Primitives::Integer(42),
        );
        flat_map.insert(
            "TestComplexStruct/simple_struct/TestSimpleStruct/simple_string/".to_string(),
            Primitives::Text("Test String".to_string()),
        );

        let mut deserializer = PrimitiveDeserializer::new(&flat_map);
        let result: TestComplexStruct = Deserialize::deserialize(&mut deserializer).unwrap();

        assert_eq!(
            result,
            TestComplexStruct {
                complex_int: 42,
                complex_string: "Test String".to_string(),
                simple_struct: TestSimpleStruct {
                    simple_int: 42,
                    simple_string: "Test String".to_string(),
                },
            }
        );
    }
}
