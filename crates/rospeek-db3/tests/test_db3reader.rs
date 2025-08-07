use std::path::Path;

use rospeek_core::BagReader;
use rospeek_db3::Db3Reader;

mod generate_db3;

#[test]
fn test_open_and_read_topics() {
    let path = Path::new("tests/data/test.db3");
    generate_db3::generate_test_db(path);

    let reader = Db3Reader::open("tests/data/test.db3").expect("Failed to open test.db3");
    let topics = reader.topics().expect("Failed to read topics");

    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].name, "/test_topic");
}

#[test]
fn test_read_messages() {
    // let path = Path::new("tests/data/test.db3");
    // generate_db3::generate_test_db(path);

    let reader = Db3Reader::open("tests/data/test.db3").expect("Failed to open test.db3");
    let messages = reader
        .read_messages("/test_topic")
        .expect("Failed to read messages");

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].timestamp, 1234567890);
    assert_eq!(
        messages[0].data,
        vec![
            0x00, 0x01, 0x00, 0x00, // CDR header
            0x06, 0x00, 0x00, 0x00, // length = 6
            b'h', b'e', b'l', b'l', b'o', 0x00, // "hello\0"
        ]
    );
}
