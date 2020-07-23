# ttlv

TTLV (tag-type-length-value) encoding scheme, which is used in KMIP (Key Management Interoperability Protocol), is a variant of the more popular TLV (tag-length-value) encoding scheme. This `#![no_std]` crate provides fast and safe TTLV encoding/decoding.

From OASIS KMIP spec v2.0:
> The scheme is designed to minimize the CPU cycle and memory requirements of clients that need to encode or decode protocol messages, and to provide optimal alignment for both 32-bit and 64-bit processors. Minimizing bandwidth over the transport mechanism is considered to be of lesser importance.

TTLV spec: <https://docs.oasis-open.org/kmip/kmip-spec/v2.0/cs01/kmip-spec-v2.0-cs01.html#_Toc6497650>

## Usage

```rust
use ttlv::{Ttlv, Value::*};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, PartialEq, Debug, FromPrimitive, ToPrimitive)]
pub enum Tag {
    Request,
    RequestHeader,
    ProtocolVersion,
    RequestBody,
}

// Construct TTLV message
let message: Ttlv = Ttlv::new(Tag::Request, Structure(vec![
    Ttlv::new(Tag::RequestHeader, Structure(vec![
        Ttlv::new(Tag::ProtocolVersion, Integer(6)),
    ])),
    Ttlv::new(Tag::RequestBody, TextString("message body")),
]));

// Encode TTLV message
let encoded = &mut [0u8; 1000];
let encoded_len = message.encode(encoded)?;

// Decode TTLV message
let (decoded, decoded_len) = Ttlv::decode(encoded)?;

// Collect data from decoded message using path
let version: i32 = decoded.path(&[Tag::RequestHeader, Tag::ProtocolVersion])?.value()?;
let message_body: &str = decoded.path(&[Tag::RequestBody])?.value()?;
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
