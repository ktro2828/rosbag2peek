#[derive(Debug, Clone)]
pub struct Topic {
    pub id: i64,
    pub name: String,
    pub type_name: String,
    pub serialization_format: String,
    pub offered_qos_profiles: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RawMessage {
    /// UNIX epoch nanoseconds
    pub timestamp: i64,
    /// Topic ID
    pub topic_id: i64,
    /// CDR-encoded message
    pub data: Vec<u8>,
}
