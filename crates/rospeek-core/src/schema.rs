use std::{env, fs::read_to_string, path::PathBuf};

use crate::error::{SchemaError, SchemaResult};

#[derive(Debug)]
pub struct MessageSchema {
    pub type_name: String,
    pub fields: Vec<MessageField>,
}

impl TryFrom<&str> for MessageSchema {
    type Error = SchemaError;

    fn try_from(type_name: &str) -> Result<Self, Self::Error> {
        let idl_path =
            find_ros_idl_path(type_name).ok_or(SchemaError::IdlNotFound(type_name.to_string()))?;
        let idl = read_to_string(idl_path)?;
        let schema = parse_idl_to_schema(&idl, type_name)?;
        Ok(schema)
    }
}

#[derive(Debug)]
pub struct MessageField {
    pub name: String,
    pub type_name: String,
    pub is_array: bool,
    pub array_len: Option<usize>,
}

pub fn find_ros_idl_path(type_name: &str) -> Option<PathBuf> {
    let mut type_name_parts = type_name.split('/');
    let package = type_name_parts.next()?;
    let msg_or_srv = type_name_parts.next()?;
    let msg_name = type_name_parts.next()?;

    if msg_or_srv != "msg" {
        return None;
    }

    let ament_paths = env::var("AMENT_PREFIX_PATH").ok()?;
    for base_path in ament_paths.split(':') {
        let candidate = PathBuf::from(base_path)
            .join("share")
            .join(package)
            .join(msg_or_srv)
            .join(format!("{msg_name}.idl"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn parse_idl_to_schema(idl: &str, type_name: &str) -> SchemaResult<MessageSchema> {
    let mut fields = Vec::new();

    let lines = idl
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && l.ends_with(";"));

    for line in lines {
        // e.g. "float64 x;" / "int32[3] values;" / "string[] names;"
        let tokens: Vec<_> = line.trim_end_matches(';').split_whitespace().collect();
        if tokens.len() != 2 {
            continue;
        }

        let (type_decl, name) = (tokens[0], tokens[1]);

        let (type_name, is_array, array_len) = if type_decl.ends_with("[]") {
            (type_decl.trim_end_matches("[]").to_string(), true, None)
        } else if let Some(start) = type_decl.find('[') {
            let base = &type_decl[..start];
            let len = &type_decl[start + 1..type_decl.len() - 1];
            (
                base.to_string(),
                true,
                Some(len.parse::<usize>().unwrap_or(0)), // unsafe fallback
            )
        } else {
            (type_decl.to_string(), false, None)
        };

        fields.push(MessageField {
            name: name.to_string(),
            type_name,
            is_array,
            array_len,
        });
    }

    Ok(MessageSchema {
        type_name: type_name.to_string(),
        fields,
    })
}
