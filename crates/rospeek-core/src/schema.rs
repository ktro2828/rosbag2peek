use std::{
    env,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::error::{RosPeekError, RosPeekResult};

#[derive(Debug, Clone)]
pub struct MessageSchema {
    /// Name of ROS message type, such ash `foo_msgs/msg/Foo`.
    pub type_name: String,
    /// Vector of message fields.
    pub fields: Vec<MessageField>,
}

impl TryFrom<&str> for MessageSchema {
    type Error = RosPeekError;

    /// Performs to try converting `type_name` into `MessageSchema` by looking up the corresponding IDL file.
    ///
    /// # Arguments
    /// * `type_name` - Name of ROS message, such as `foo_msgs/msg/Foo`.
    ///
    /// # Examples
    /// ```
    /// let schema = rospeek_core::MessageSchema::try_from("std_msgs/msg/Float64").unwrap();
    /// ```
    fn try_from(type_name: &str) -> Result<Self, Self::Error> {
        let idl =
            find_ros_idl_path(type_name).ok_or(RosPeekError::IdlNotFound(type_name.to_string()))?;
        let schema = parse_idl_to_schema(idl, type_name)?;
        Ok(schema)
    }
}

#[derive(Debug, Clone)]
pub struct MessageField {
    /// Name of field.
    pub name: String,
    /// Field type
    pub field_type: FieldType,
}

impl MessageField {
    pub fn type_name(&self) -> &str {
        self.field_type.type_name()
    }

    pub fn is_iterable(&self) -> bool {
        self.field_type.is_iterable()
    }
}

#[derive(Debug, Clone)]
pub enum FieldType {
    /// Primitive or nested structures, where `T`: (type_name)
    Object(String),
    /// Sequential objects, where `sequence<T>`: (type_name)
    Sequence(String),
    /// Array objects, where `T[N]`: (type_name, length)
    Array(String, usize),
}

impl FieldType {
    fn type_name(&self) -> &str {
        match self {
            FieldType::Object(n) => n,
            FieldType::Sequence(n) => n,
            FieldType::Array(n, _) => n,
        }
    }

    fn is_iterable(&self) -> bool {
        !matches!(self, FieldType::Object(_))
    }
}

/// Performs to try looking up the corresponding IDL file.
///
/// # Arguments
/// * `type_name` - Name of ROS message type, such as `foo_msgs/msg/Foo`.
///
/// # Examples
/// ```
/// let path = rospeek_core::find_ros_idl_path("std_msgs/msg/Float64").unwrap();
///
/// let expect = rospeek_core::read_to_filepath("/opt/ros/$ROS_DISTRO/share/std_msgs/msg/Float64.idl").unwrap();
/// assert_eq!(path, expect);
/// ```
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

/// Performs to try parsing IDL and convert to `MessageSchema`.
///
/// # Arguments
/// * `idl` - IDL definition in string.
/// * `type_name` - Name of ROS message type, such as `foo_msgs/msg/Foo`.
///
/// # Examples
/// ```
/// let schema = rospeek_core::parse_idl_to_schema("/opt/ros/$ROS_DISTRO/share/std_msgs/msg/Float64.idl", "std_msgs/msg/Float64").unwrap();
///
/// assert_eq!(schema.type_name, "std_msgs/msg/Float64".to_string());
/// assert_eq!(schema.fields.len(), 1);
/// assert_eq!(schema.fields[0].name, "data".to_string());
/// assert_eq!(schema.fields[0].type_name(), "double".to_string());
/// ```
pub fn parse_idl_to_schema<P: AsRef<Path>>(
    idl: P,
    type_name: &str,
) -> RosPeekResult<MessageSchema> {
    let idl_path = read_to_filepath(idl.as_ref())?;
    let idl_str = read_to_string(&idl_path)?;

    let lines = idl_str
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && l.ends_with(";"));

    let mut fields = Vec::new();
    for line in lines {
        // e.g. "float64 x;" / "int32[3] values;" / "string[] names;"
        let tokens: Vec<_> = line.trim_end_matches(';').split_whitespace().collect();
        if tokens.len() != 2 {
            continue;
        }

        let (type_decl, name) = (tokens[0], tokens[1]);

        let field_type = to_field_type(type_decl);

        fields.push(MessageField {
            name: name.to_string(),
            field_type,
        });
    }

    Ok(MessageSchema {
        type_name: type_name.to_string(),
        fields,
    })
}

/// Reads a file path from a string, expanding any shell variables.
///
/// # Arguments
/// * `path` - The path to the file to read.
///
/// # Returns
/// A `RosPeekResult` containing the path to the file.
///
/// # Examples
/// ```
/// let filepath = rospeek_core::read_to_filepath("$HOME/Documents/example.txt").unwrap();
/// println!("Filepath: {:?}", filepath);
/// ```
pub fn read_to_filepath<P: AsRef<Path>>(path: P) -> RosPeekResult<PathBuf> {
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| RosPeekError::Other("Path contains invalid UTF-8".to_string()))?;
    let expanded = shellexpand::full(path_str).map_err(|e| RosPeekError::Other(e.to_string()))?;
    Ok(PathBuf::from(expanded.as_ref()))
}

fn to_field_type(s: &str) -> FieldType {
    let re = Regex::new(r"^(?P<type>.+)__(?P<num>\d+)$").unwrap();
    if let Some(capture) = re.captures(s) {
        let type_name = capture.name("type").unwrap().as_str().replace("::", "/");
        let num = capture
            .name("num")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap_or(0);
        FieldType::Array(type_name, num)
    } else if s.starts_with("sequence<") && s.ends_with(">") {
        let type_name = &s["sequence<".len()..s.len() - 1];
        FieldType::Sequence(type_name.replace("::", "/"))
    } else {
        FieldType::Object(s.replace("::", "/"))
    }
}
