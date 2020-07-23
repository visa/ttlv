// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
extern crate alloc;

mod ttlv;
mod util;

pub use crate::ttlv::*;
pub use crate::util::parse_ttlv_len;

#[cfg(test)]
mod tests {
    use super::{Value::*, *};
    use alloc::vec;
    use num_derive::{FromPrimitive, ToPrimitive};

    #[derive(Copy, Clone, PartialEq, Debug, FromPrimitive, ToPrimitive)]
    pub enum Tag {
        Request,
        RequestHeader,
        ProtocolVersion,
        RequestBody,
    }

    #[test]
    fn encode_decode() -> Result<(), Error> {
        // Construct TTLV message
        let message: Ttlv = Ttlv::new(
            Tag::Request,
            Structure(vec![
                Ttlv::new(
                    Tag::RequestHeader,
                    Structure(vec![Ttlv::new(Tag::ProtocolVersion, Integer(6))]),
                ),
                Ttlv::new(Tag::RequestBody, TextString("message body")),
            ]),
        );

        // Encode TTLV message
        let encoded = &mut [0u8; 1000];
        let encoded_len = message.encode(encoded)?;

        // Decode TTLV message
        let (decoded, decoded_len) = Ttlv::decode(encoded)?;
        assert_eq!(encoded_len, decoded_len);
        assert_eq!(message, decoded);

        // Collect data from decoded message using path
        let version: i32 = decoded
            .path(&[Tag::RequestHeader, Tag::ProtocolVersion])?
            .value()?;
        assert_eq!(6, version);
        let message_body: &str = decoded.path(&[Tag::RequestBody])?.value()?;
        assert_eq!("message body", message_body);
        Ok(())
    }
}
