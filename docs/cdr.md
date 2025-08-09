# Common Data Representation (CDR)

## What is CDR?

CDR is a binary serialization format defined by the OMG (Object Management Group). It is used to serialize and deserialize data defined in the IDL (Interface Definition Language) specification.

ROS 2 uses CDR (specifically _XCDR1_) via its DDS middleware to transmit and store messages.

## Serialized Payload Representation

Serialized CDR payload in DDS/ROS 2 typically consists of:

```css
0........................4..................... bytes
| [Encapsulation Header] | [Serialized Body]
```

### Encapsulation Header (CDR Header)

The **Encapsulation Header** is a 4-byte prefix defined by the OMG DDS-XTypes specification.
It indicates the endianness and format of the serialized data.

```css
0...2...........8...............16..............24..............32
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   representation_identifier   |    representation_options     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~                                                               ~
~ ... Bytes of data representation using a format that ...      ~
~ ... depends on the RepresentationIdentifier and options ...   ~
~                                                               ~
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

| Offset | Size | Field                    | Description                                                     |
| ------ | ---- | ------------------------ | --------------------------------------------------------------- |
| 0      | 2B   | Encapsulation Identifier | Format & endianness: e.g., `0x0000` = CDR_BE, `0x0001` = CDR_LE |
| 2      | 2B   | Options/Unused           | Usually `0x0000` in ROS 2 XCDR1                                 |

### Serialized Body

The **Serialized Body** follows the encapsulation header and contains the actual serialized data, encoded according to the CDR rules:

- Each field is serialized in the order defined in the IDL.
- Alignment and padding rules apply
- Variable-length types (e.g., `string`, `sequence`) are prefixed by their length in `uint32`.

#### Basic Types in IDL

About basic type definitions in IDL, please refer to [IDL - Interface Definition and Language Mapping](https://design.ros2.org/articles/idl_interface_definition.html).

The following table shows the mapping between IDL types and Rust types for basic types:

| IDL Type  | Rust Type | Size (Alignment) |
| --------- | --------- | ---------------- |
| `boolean` | `bool`    | 1 byte           |
| `octet`   | `u8`      | 1 byte           |
| `char`    | `char`    | 1 byte           |
| `int8`    | `i8`      | 1 byte           |
| `int16`   | `i16`     | 2 bytes          |
| `int32`   | `i32`     | 4 bytes          |
| `int64`   | `i64`     | 8 bytes          |
| `uint8`   | `u8`      | 1 byte           |
| `uint16`  | `u16`     | 2 bytes          |
| `uint32`  | `u32`     | 4 bytes          |
| `uint64`  | `u64`     | 8 bytes          |
| `float`   | `f32`     | 4 bytes          |
| `double`  | `f64`     | 8 bytes          |

For example, the definition of `Point` in IDL and its binary representation are as follows:

```txt
/// IDL Point
module Point {
  double x;
  double y;
  double z;
};

/// Binary representation of `Point`
0x00 0x01 0x00 0x00   // Encapsulation header (Little Endian)
0x00 0x00 0x80 3F     // x = 1.0
0x00 0x00 0x00 40     // y = 2.0
0x00 0x00 0x40 40     // z = 3.0
```

The following table shows the mapping between IDL types and Rust types for sequential types:

| IDL Type      | Rust Type | Size (Alignment) |
| ------------- | --------- | ---------------- |
| `T__N`        | `[T; N]`  | N \* SizeOf(T)   |
| `sequence<N>` | `Vec<T>`  | N \* SizeOf(T)   |
| `string`      | `String`  | N \* 1 bytes     |

For fixed array types, the length is described as `T__N` in IDL as follows:

```txt
module Array {
  double__3 value;
};

/// Binary representation of `Array`
0x00 0x01 0x00 0x00    // Encapsulation header (Little Endian)
0x00 0x00 0x80 0x3F    // a[0] = 1.0
0x00 0x00 0x00 0x40    // a[1] = 2.0
0x00 0x00 0x40 0x40    // a[2] = 3.0
```

For sequential types, the length is stored as a 4-byte unsigned integer before the actual data:

```txt
/// Binary representation of `string`
0x00, 0x01, 0x00, 0x00,             // Encapsulation header (Little Endian)
0x06, 0x00, 0x00, 0x00,             // N = 6
b'h', b'e', b'l', b'l', b'o', 0x00, // "hello\0"
```
