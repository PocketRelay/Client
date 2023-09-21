use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::Debug;
use std::io;
use tdf::{serialize_vec, TdfSerialize};
use tokio_util::codec::{Decoder, Encoder};

/// The different types of packets
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    /// ID counted request packets (0x00)
    Request = 0x00,
    /// Packets responding to requests (0x10)
    Response = 0x10,
    /// Unique packets coming from the server (0x20)
    Notify = 0x20,
    /// Error packets (0x30)
    Error = 0x30,
}

/// From u8 implementation to convert bytes back into
/// PacketTypes
impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => PacketType::Request,
            0x10 => PacketType::Response,
            0x20 => PacketType::Notify,
            0x30 => PacketType::Error,
            _ => PacketType::Request,
        }
    }
}

/// Structure of packet header which comes before the
/// packet content and describes it.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    /// The component of this packet
    pub component: u16,
    /// The command of this packet
    pub command: u16,
    /// A possible error this packet contains (zero is none)
    pub error: u16,
    /// The type of this packet
    pub ty: PacketType,
    /// The unique ID of this packet (Notify packets this is just zero)
    pub id: u16,
}

impl PacketHeader {
    /// Creates a response to the provided packet header by
    /// changing the type of the header
    pub const fn response(&self) -> Self {
        self.with_type(PacketType::Response)
    }

    /// Copies the header contents changing its Packet Type
    ///
    /// `ty` The new packet type
    pub const fn with_type(&self, ty: PacketType) -> Self {
        Self {
            component: self.component,
            command: self.command,
            error: self.error,
            ty,
            id: self.id,
        }
    }

    pub fn write(&self, dst: &mut BytesMut, length: usize) {
        let is_extended = length > 0xFFFF;
        dst.put_u16(length as u16);
        dst.put_u16(self.component);
        dst.put_u16(self.command);
        dst.put_u16(self.error);
        dst.put_u8(self.ty as u8);
        dst.put_u8(if is_extended { 0x10 } else { 0x00 });
        dst.put_u16(self.id);
        if is_extended {
            dst.put_u8(((length & 0xFF000000) >> 24) as u8);
            dst.put_u8(((length & 0x00FF0000) >> 16) as u8);
        }
    }

    pub fn read(src: &mut BytesMut) -> Option<(PacketHeader, usize)> {
        if src.len() < 12 {
            return None;
        }

        let mut length = src.get_u16() as usize;
        let component = src.get_u16();
        let command = src.get_u16();
        let error = src.get_u16();
        let ty = src.get_u8();
        // If we encounter 0x10 here then the packet contains extended length
        // bytes so its longer than a u16::MAX length
        let is_extended = src.get_u8() == 0x10;
        let id = src.get_u16();

        if is_extended {
            // We need another two bytes for the extended length
            if src.len() < 2 {
                return None;
            }
            length += src.get_u16() as usize;
        }

        let ty = PacketType::from(ty);
        let header = PacketHeader {
            component,
            command,
            error,
            ty,
            id,
        };
        Some((header, length))
    }
}

/// Structure for Blaze packets contains the contents of the packet
/// and the header for identification.
///
/// Packets can be cloned with little memory usage increase because
/// the content is stored as Bytes.
#[derive(Debug, Clone)]
pub struct Packet {
    /// The packet header
    pub header: PacketHeader,
    /// The packet encoded byte contents
    pub contents: Bytes,
}

fn serialize_bytes<V>(value: &V) -> Bytes
where
    V: TdfSerialize,
{
    Bytes::from(serialize_vec(value))
}

impl Packet {
    /// Creates a new packet from the provided header and contents
    pub const fn new(header: PacketHeader, contents: Bytes) -> Self {
        Self { header, contents }
    }

    /// Creates a new packet from the provided header with empty content
    #[inline]
    pub const fn new_empty(header: PacketHeader) -> Self {
        Self::new(header, Bytes::new())
    }

    #[inline]
    pub const fn new_response(packet: &Packet, contents: Bytes) -> Self {
        Self::new(packet.header.response(), contents)
    }

    #[inline]
    pub const fn response_empty(packet: &Packet) -> Self {
        Self::new_empty(packet.header.response())
    }

    #[inline]
    pub fn response<V>(packet: &Packet, contents: V) -> Self
    where
        V: TdfSerialize,
    {
        Self::new_response(packet, serialize_bytes(&contents))
    }

    pub fn read(src: &mut BytesMut) -> Option<Self> {
        let (header, length) = PacketHeader::read(src)?;

        if src.len() < length {
            return None;
        }

        let contents = src.split_to(length);
        Some(Self {
            header,
            contents: contents.freeze(),
        })
    }

    pub fn write(&self, dst: &mut BytesMut) {
        let contents = &self.contents;
        self.header.write(dst, contents.len());
        dst.extend_from_slice(contents);
    }
}

/// Tokio codec for encoding and decoding packets
pub struct PacketCodec;

impl Decoder for PacketCodec {
    type Error = io::Error;
    type Item = Packet;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut read_src = src.clone();
        let result = Packet::read(&mut read_src);

        if result.is_some() {
            *src = read_src;
        }

        Ok(result)
    }
}

impl Encoder<Packet> for PacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.write(dst);
        Ok(())
    }
}
