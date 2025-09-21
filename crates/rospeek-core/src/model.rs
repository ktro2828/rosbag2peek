#[derive(Debug, Clone)]
pub struct Topic {
    pub id: u16,
    pub name: String,
    pub type_name: String,
    pub count: u64,
    pub serialization_format: String,
    pub offered_qos_profiles: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RawMessage {
    /// UNIX epoch nanoseconds
    pub timestamp: u64,
    /// Topic ID
    pub topic_id: u16,
    /// CDR-encoded message
    pub data: Vec<u8>,
}
