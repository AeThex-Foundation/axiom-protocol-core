use crate::Message;
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024; // 8 MiB hard limit

/// Length-prefixed bincode codec for [`Message`].
///
/// Wire format: `[ u32 length (LE) ][ bincode payload ]`
#[derive(Debug, Default)]
pub struct AxiomCodec;

impl Encoder<Message> for AxiomCodec {
    type Error = std::io::Error;

    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload = bincode::serialize(&msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if payload.len() > MAX_FRAME_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("message too large: {} bytes", payload.len()),
            ));
        }

        dst.reserve(4 + payload.len());
        dst.put_u32_le(payload.len() as u32);
        dst.put_slice(&payload);
        Ok(())
    }
}

impl Decoder for AxiomCodec {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        let len = u32::from_le_bytes([src[0], src[1], src[2], src[3]]) as usize;

        if len > MAX_FRAME_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("incoming frame too large: {len} bytes"),
            ));
        }

        if src.len() < 4 + len {
            src.reserve(4 + len - src.len());
            return Ok(None);
        }

        src.advance(4);
        let payload = src.split_to(len);

        let msg = bincode::deserialize(&payload)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Some(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn roundtrip_ping() {
        let mut codec = AxiomCodec;
        let mut buf = BytesMut::new();
        codec.encode(Message::Ping { nonce: 42 }, &mut buf).unwrap();
        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert!(matches!(decoded, Message::Ping { nonce: 42 }));
    }

    #[test]
    fn partial_frame_returns_none() {
        let mut codec = AxiomCodec;
        let mut buf = BytesMut::new();
        codec.encode(Message::Ping { nonce: 1 }, &mut buf).unwrap();
        // Remove the last byte to simulate partial read
        buf.truncate(buf.len() - 1);
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }
}
