use std::io::{Cursor, Read};

use serde_json::json;

use crate::{
    FieldType, MessageField, MessageSchema,
    error::{SchemaError, SchemaResult},
};

pub struct CdrDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> CdrDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn decode(&mut self, schema: &MessageSchema) -> SchemaResult<serde_json::Value> {
        let mut object = serde_json::Map::new();

        for field in schema.fields.iter() {
            let value = if field.is_iterable() {
                self.decode_iterable(field)?
            } else {
                self.decode_primitive(field)?
            };
            object.insert(field.name.clone(), value);
        }
        Ok(serde_json::Value::Object(object))
    }

    fn decode_primitive(&mut self, field: &MessageField) -> SchemaResult<serde_json::Value> {
        // NOTE: https://design.ros2.org/articles/idl_interface_definition.html
        // TODO: [wchar, wstring] is not supported yet
        match field.type_name() {
            "boolean" => Ok(json!(self.decode_bool()?)),
            "octet" => Ok(json!(self.decode_u8()?)),
            "char" => Ok(json!(self.decode_char()?)),
            "float" => Ok(json!(self.decode_f32()?)),
            "double" => Ok(json!(self.decode_f64()?)),
            "int8" => Ok(json!(self.decode_i8()?)),
            "uint8" => Ok(json!(self.decode_u8()?)),
            "int16" => Ok(json!(self.decode_i16()?)),
            "uint16" => Ok(json!(self.decode_u16()?)),
            "int32" => Ok(json!(self.decode_i32()?)),
            "uint32" => Ok(json!(self.decode_u32()?)),
            "int64" => Ok(json!(self.decode_i64()?)),
            "uint64" => Ok(json!(self.decode_u64()?)),
            "string" => Ok(json!(self.decode_string()?)),
            // === special ROS 2 types ===
            "builtin_interfaces/msg/Time/Time"
            | "builtin_interfaces/Time"
            | "builtin_interfaces/msg/Duration"
            | "builtin_interfaces/Duration" => {
                let sec = self.decode_i32()?;
                let nanosec = self.decode_u32()?;
                Ok(json!({"sec": sec, "nanosec": nanosec}))
            }
            // === nested structures ===
            _ => {
                let nested_schema = MessageSchema::try_from(field.type_name())?;
                self.decode(&nested_schema)
            }
        }
    }

    fn decode_iterable(&mut self, field: &MessageField) -> SchemaResult<serde_json::Value> {
        let length = match field.field_type {
            FieldType::Array(_, n) => n,
            FieldType::Sequence(_) => self.decode_u32()? as usize,
            _ => 0,
        };

        let mut items = Vec::with_capacity(length);
        for _ in 0..length {
            items.push(self.decode_primitive(field)?);
        }
        Ok(json!(items))
    }

    // === Decode methods for each primitive ===

    fn decode_bytes<const N: usize>(&mut self) -> SchemaResult<[u8; N]> {
        let mut buf = [0u8; N];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn decode_bool(&mut self) -> SchemaResult<bool> {
        let b = self.decode_u8()?;
        Ok(b != 0)
    }

    fn decode_u8(&mut self) -> SchemaResult<u8> {
        let mut buf = [0u8; 1];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn decode_i8(&mut self) -> SchemaResult<i8> {
        self.decode_u8().map(|v| v as i8)
    }

    fn decode_u16(&mut self) -> SchemaResult<u16> {
        let buf = self.decode_bytes::<2>()?;
        Ok(u16::from_le_bytes(buf))
    }

    fn decode_i16(&mut self) -> SchemaResult<i16> {
        let buf = self.decode_bytes::<2>()?;
        Ok(i16::from_le_bytes(buf))
    }

    fn decode_u32(&mut self) -> SchemaResult<u32> {
        let buf = self.decode_bytes::<4>()?;
        Ok(u32::from_le_bytes(buf))
    }

    fn decode_i32(&mut self) -> SchemaResult<i32> {
        let buf = self.decode_bytes::<4>()?;
        Ok(i32::from_le_bytes(buf))
    }

    fn decode_u64(&mut self) -> SchemaResult<u64> {
        let buf = self.decode_bytes::<8>()?;
        Ok(u64::from_le_bytes(buf))
    }

    fn decode_i64(&mut self) -> SchemaResult<i64> {
        let buf = self.decode_bytes::<8>()?;
        Ok(i64::from_le_bytes(buf))
    }

    fn decode_f32(&mut self) -> SchemaResult<f32> {
        let buf = self.decode_bytes::<4>()?;
        Ok(f32::from_le_bytes(buf))
    }

    fn decode_f64(&mut self) -> SchemaResult<f64> {
        let buf = self.decode_bytes::<8>()?;
        Ok(f64::from_le_bytes(buf))
    }

    fn decode_char(&mut self) -> SchemaResult<char> {
        let code = self.decode_u8()?; // char in IDL is ASCII
        Ok(code as char)
    }

    fn decode_string(&mut self) -> SchemaResult<String> {
        let len = self.decode_u32()? as usize;
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        if buf.last() == Some(&0) {
            buf.pop(); // null terminator (optional in ROS2)
        }
        String::from_utf8(buf).map_err(|e| SchemaError::InvalidData(e))
    }
}
