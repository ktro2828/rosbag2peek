use rospeek_core::{CdrDecoder, MessageField, MessageSchema};

#[test]
fn test_decode_time() {
    let data = [
        0xD2, 0x02, 0x00, 0x00, // sec = 722
        0x88, 0x13, 0x00, 0x00, // nanosec = 5000
    ];

    let type_name = String::from("builtin_interfaces/msg/Time");

    // if IDL file found use it, otherwise initialize manually
    let schema = MessageSchema::try_from(type_name.as_ref()).unwrap_or_else(|_| MessageSchema {
        type_name: type_name.clone(),
        fields: vec![
            MessageField {
                name: "sec".into(),
                type_name: "int32".into(),
                is_array: false,
                array_len: None,
            },
            MessageField {
                name: "nanosec".into(),
                type_name: "uint32".into(),
                is_array: false,
                array_len: None,
            },
        ],
    });

    let mut decoder = CdrDecoder::new(&data);
    let result = decoder
        .decode(&schema)
        .expect(format!("Failed to decode {type_name}").as_ref());
    assert_eq!(result["sec"], 722);
    assert_eq!(result["nanosec"], 5000);
}

#[test]
fn test_decode_duration() {
    let data = [
        0xD2, 0x02, 0x00, 0x00, // sec = 722
        0x88, 0x13, 0x00, 0x00, // nanosec = 5000
    ];

    let type_name = String::from("builtin_interfaces/msg/Duration");

    // if IDL file found use it, otherwise initialize manually
    let schema = MessageSchema::try_from(type_name.as_ref()).unwrap_or_else(|_| MessageSchema {
        type_name: type_name.clone(),
        fields: vec![
            MessageField {
                name: "sec".into(),
                type_name: "int32".into(),
                is_array: false,
                array_len: None,
            },
            MessageField {
                name: "nanosec".into(),
                type_name: "uint32".into(),
                is_array: false,
                array_len: None,
            },
        ],
    });

    let mut decoder = CdrDecoder::new(&data);
    let result = decoder
        .decode(&schema)
        .expect(format!("Failed to decode {type_name}").as_ref());
    assert_eq!(result["sec"], 722);
    assert_eq!(result["nanosec"], 5000);
}

#[test]
fn test_decode_string_array() {
    let data = [
        0x02, 0x00, 0x00, 0x00, // array length = 2
        0x06, 0x00, 0x00, 0x00, // string length = 6
        b'h', b'e', b'l', b'l', b'o', 0x00, // "hello\\0"
        0x06, 0x00, 0x00, 0x00, // string length = 6
        b'w', b'o', b'r', b'l', b'd', 0x00, // "world\\0"
    ];

    let type_name = String::from("custom/msg/StringArray");
    let schema = MessageSchema {
        type_name: type_name.clone(),
        fields: vec![MessageField {
            name: "values".to_string(),
            type_name: "string".to_string(),
            is_array: true,
            array_len: None,
        }],
    };

    let mut decoder = CdrDecoder::new(&data);
    let result = decoder
        .decode(&schema)
        .expect(format!("Failed to decode {type_name}").as_ref());
    assert_eq!(result["values"][0], "hello");
    assert_eq!(result["values"][1], "world");
}
