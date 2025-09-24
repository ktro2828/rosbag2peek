use std::{
    collections::{BTreeSet, HashMap},
    io::{Cursor, Read},
    sync::Arc,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::{Value, json};

use crate::{
    BagReader, FieldType, MessageField, MessageSchema,
    error::{RosPeekError, RosPeekResult},
    flatten_json,
};

#[derive(Debug)]
enum Endianness {
    Little,
    Big,
}

pub struct CdrDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
    endianness: Endianness,
    cache: HashMap<String, Arc<MessageSchema>>,
}

fn to_endianness(data: &[u8]) -> Endianness {
    match data.get(1).copied().unwrap_or(0x01) {
        0x00 => Endianness::Big,
        0x01 => Endianness::Little,
        _ => Endianness::Little,
    }
}

impl<'a> CdrDecoder<'a> {
    /// Creates a new decoder from CDR-encoded data.
    ///
    /// # Arguments
    /// * `data` - CDR-encoded data
    pub fn new(data: &'a [u8]) -> Self {
        // determine endianness by checking the second byte
        let endianness = to_endianness(data);

        Self {
            cursor: Cursor::new(&data[4..]), // first 4bytes are header, so skip them
            endianness,
            cache: HashMap::new(),
        }
    }

    /// Creates a new decoder from schema.
    ///
    /// # Arguments
    /// * `schema` - ROS message schema
    pub fn from_schema(schema: &MessageSchema) -> Self {
        let mut cache = HashMap::new();
        cache.insert(schema.type_name.clone(), Arc::new(schema.clone()));
        Self {
            cursor: Cursor::new(&[]),
            endianness: Endianness::Little,
            cache,
        }
    }

    /// Resets cursor from CDR-encoded data.
    ///
    /// # Arguments
    /// * `data` - CDR-encoded data
    pub fn reset(&mut self, data: &'a [u8]) -> &mut Self {
        self.endianness = to_endianness(data);
        self.cursor = Cursor::new(&data[4..]);
        self
    }

    /// Performs decoding CDR-encoded data for the corresponding schema.
    ///
    /// # Arguments
    /// * `schema` - ROS message schema
    pub fn decode(&mut self, schema: &MessageSchema) -> RosPeekResult<serde_json::Value> {
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

    fn decode_primitive(&mut self, field: &MessageField) -> RosPeekResult<serde_json::Value> {
        match field.type_name() {
            // === primitive types ===
            // NOTE: https://design.ros2.org/articles/idl_interface_definition.html
            // TODO(ktro2828): [wchar, wstring] is not supported yet
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
            "builtin_interfaces/msg/Time" | "builtin_interfaces/msg/Duration" => {
                let sec = self.decode_i32()?;
                let nanosec = self.decode_u32()?;
                Ok(json!({"sec": sec, "nanosec": nanosec}))
            }
            // === nested structures ===
            _ => {
                let nested_schema = self.get_schema(field.type_name())?;
                self.decode(nested_schema.as_ref())
            }
        }
    }

    fn decode_iterable(&mut self, field: &MessageField) -> RosPeekResult<serde_json::Value> {
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

    fn get_schema(&mut self, type_name: &str) -> RosPeekResult<Arc<MessageSchema>> {
        if !self.cache.contains_key(type_name) {
            let schema = Arc::new(MessageSchema::try_from(type_name)?);
            self.cache.insert(type_name.to_string(), schema);
        }
        Ok(self.cache.get(type_name).unwrap().clone())
    }

    // === Decode methods for each primitive ===

    fn align_to(&mut self, align: usize) -> RosPeekResult<()> {
        let pos = self.cursor.position() as usize;
        let padding = (align - (pos % align)) % align;
        if padding > 0 {
            let mut buf = vec![0u8; padding];
            self.cursor.read_exact(&mut buf)?;
        }
        Ok(())
    }

    fn decode_bytes<const N: usize>(&mut self) -> RosPeekResult<[u8; N]> {
        let mut buf = [0u8; N];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn decode_bool(&mut self) -> RosPeekResult<bool> {
        let b = self.decode_u8()?;
        Ok(b != 0)
    }

    fn decode_u8(&mut self) -> RosPeekResult<u8> {
        let mut buf = [0u8; 1];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn decode_i8(&mut self) -> RosPeekResult<i8> {
        self.decode_u8().map(|v| v as i8)
    }

    fn decode_u16(&mut self) -> RosPeekResult<u16> {
        self.align_to(2)?;
        let buf = self.decode_bytes::<2>()?;
        Ok(match self.endianness {
            Endianness::Big => u16::from_be_bytes(buf),
            Endianness::Little => u16::from_le_bytes(buf),
        })
    }

    fn decode_i16(&mut self) -> RosPeekResult<i16> {
        self.align_to(2)?;
        let buf = self.decode_bytes::<2>()?;
        Ok(match self.endianness {
            Endianness::Big => i16::from_be_bytes(buf),
            Endianness::Little => i16::from_le_bytes(buf),
        })
    }

    fn decode_u32(&mut self) -> RosPeekResult<u32> {
        self.align_to(4)?;
        let buf = self.decode_bytes::<4>()?;
        Ok(match self.endianness {
            Endianness::Big => u32::from_be_bytes(buf),
            Endianness::Little => u32::from_le_bytes(buf),
        })
    }

    fn decode_i32(&mut self) -> RosPeekResult<i32> {
        self.align_to(4)?;
        let buf = self.decode_bytes::<4>()?;
        Ok(match self.endianness {
            Endianness::Big => i32::from_be_bytes(buf),
            Endianness::Little => i32::from_le_bytes(buf),
        })
    }

    fn decode_u64(&mut self) -> RosPeekResult<u64> {
        self.align_to(8)?;
        let buf = self.decode_bytes::<8>()?;
        Ok(match self.endianness {
            Endianness::Big => u64::from_be_bytes(buf),
            Endianness::Little => u64::from_le_bytes(buf),
        })
    }

    fn decode_i64(&mut self) -> RosPeekResult<i64> {
        self.align_to(8)?;
        let buf = self.decode_bytes::<8>()?;
        Ok(match self.endianness {
            Endianness::Big => i64::from_be_bytes(buf),
            Endianness::Little => i64::from_le_bytes(buf),
        })
    }

    fn decode_f32(&mut self) -> RosPeekResult<f32> {
        self.align_to(4)?;
        let buf = self.decode_bytes::<4>()?;
        Ok(match self.endianness {
            Endianness::Big => f32::from_be_bytes(buf),
            Endianness::Little => f32::from_le_bytes(buf),
        })
    }

    fn decode_f64(&mut self) -> RosPeekResult<f64> {
        self.align_to(8)?;
        let buf = self.decode_bytes::<8>()?;
        Ok(match self.endianness {
            Endianness::Big => f64::from_be_bytes(buf),
            Endianness::Little => f64::from_le_bytes(buf),
        })
    }

    fn decode_char(&mut self) -> RosPeekResult<char> {
        let code = self.decode_u8()?; // char in IDL is ASCII
        Ok(code as char)
    }

    fn decode_string(&mut self) -> RosPeekResult<String> {
        let len = self.decode_u32()? as usize;
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        if buf.last() == Some(&0) {
            buf.pop(); // null terminator (optional in ROS 2)
        }
        String::from_utf8(buf).map_err(RosPeekError::InvalidUtf8)
    }
}

/// Decodes messages for a given topic into JSON parallel.
///
/// # Arguments
/// * `reader` - The bag reader to read messages from.
/// * `topic` - The topic to decode messages for.
///
/// # Returns
/// A vector of JSON values representing the decoded messages.
pub fn try_decode_json(
    reader: Box<dyn BagReader>,
    topic: &str,
) -> RosPeekResult<Vec<serde_json::Value>> {
    let topic_info = reader
        .topics()?
        .into_iter()
        .find(|t| t.name == topic)
        .ok_or_else(|| RosPeekError::TopicNotFound(topic.to_string()))?;

    let schema = Arc::new(MessageSchema::try_from(topic_info.type_name.as_ref())?);

    let messages = reader.read_messages(topic)?;

    let values = messages
        .par_iter()
        .map_init(
            || CdrDecoder::from_schema(&schema),
            |decoder, msg| decoder.reset(&msg.data).decode(&schema),
        )
        .collect::<RosPeekResult<Vec<_>>>()?;

    Ok(values)
}

/// Decode a topic into a CSV format.
///
/// # Arguments
/// * `reader` - The bag reader to read messages from.
/// * `topic` - The topic to decode.
///
/// # Returns
/// A tuple containing the column names and rows of the decoded CSV.
pub fn try_decode_csv(
    reader: Box<dyn BagReader>,
    topic: &str,
) -> RosPeekResult<(BTreeSet<String>, Vec<Vec<String>>)> {
    let json_values = try_decode_json(reader, topic)?;

    let mut columns = BTreeSet::new();
    let mut rows = Vec::with_capacity(json_values.len());
    for value in json_values {
        if let Value::Object(object) = value {
            let result = flatten_json(&object)?;
            columns.extend(result.keys().cloned());
            let row = columns
                .iter()
                .map(|col| result.get(col).map(|v| v.to_string()).unwrap_or_default())
                .collect();
            rows.push(row);
        }
    }
    Ok((columns, rows))
}

pub fn try_decode_binary<'a>(
    decoder: &mut CdrDecoder<'a>,
    schema: &MessageSchema,
    data: &'a [u8],
) -> RosPeekResult<String> {
    let value = decoder.reset(data).decode(schema)?;

    serde_json::to_string_pretty(&value)
        .map_err(|e| RosPeekError::Other(format!("Failed to format JSON: {}", e)))
}
