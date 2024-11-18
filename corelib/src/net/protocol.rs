#![allow(unused)]
use std::io::Write;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::errors::{Error, ProtocolError, Result};

use super::message::{deserialize, serialize, Message};

#[derive(Default)]
pub enum SupportedVersions {
    #[default]
    One = 1,
}

pub const VERSION: SupportedVersions = SupportedVersions::One;

impl SupportedVersions {
    pub fn as_u16(&self) -> u16 {
        match self {
            Self::One => 1,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize, PartialEq, Eq)]
#[borsh(use_discriminant = true)]
pub enum Command {
    Ping = 1,
    Get = 2,
    Post = 3,
}

impl TryFrom<u8> for Command {
    type Error = ProtocolError;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Command::Ping),
            2 => Ok(Command::Get),
            3 => Ok(Command::Post),
            n => Err(ProtocolError::UnsupportedCommand(n)),
        }
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Header {
    version: u16,
    content_size: u16,
}

impl Header {
    pub fn new(content_size: u16) -> Self {
        Header {
            version: VERSION.as_u16(),
            content_size,
        }
    }

    pub fn to_bytes(&self, buffer: &mut Vec<u8>) -> Result<()> {
        buffer.write_all(&self.version.to_be_bytes())?;
        buffer.write_all(&self.content_size.to_be_bytes())?;
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(Error::Protocol(ProtocolError::InvalidMessageFormat));
        }

        let version = u16::from_be_bytes([bytes[0], bytes[1]]);
        let content_size = u16::from_be_bytes([bytes[2], bytes[3]]);

        if version != VERSION.as_u16() {
            return Err(Error::Protocol(ProtocolError::UnknownVersion(version)));
        }

        Ok(Header {
            version,
            content_size,
        })
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Request {
    header: Header,
    command: Command,
    payload: Option<Message>,
}

impl Request {
    pub fn new(command: Command, payload: Option<Message>) -> Result<Self> {
        let content_size = if let Some(ref p) = payload {
            let mut serialized_payload = Vec::new();
            serialize(p, &mut serialized_payload)?;
            serialized_payload.len() as u16
        } else {
            0
        };
        let header = Header::new(content_size);
        Ok(Request {
            header,
            command,
            payload,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        write_to_buffer(
            &self.header,
            &self.command,
            self.payload.as_ref(),
            &mut buffer,
        )?;

        Ok(buffer)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (header, command, payload) = read_from_buffer::<Command>(bytes)?;

        Ok(Request {
            header,
            command,
            payload,
        })
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn payload(&self) -> &Option<Message> {
        &self.payload
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum StatusCode {
    OK = 0,
    NotFound = 1,
    Error = 2,
}

impl TryFrom<u8> for StatusCode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> std::result::Result<Self, <StatusCode as TryFrom<u8>>::Error> {
        match value {
            0 => Ok(StatusCode::OK),
            1 => Ok(StatusCode::NotFound),
            2 => Ok(StatusCode::Error),
            n => Err(ProtocolError::UnsupportedStatusCode(n)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    header: Header,
    status: StatusCode,
    payload: Option<Message>,
}

impl Response {
    pub fn new(status: StatusCode, payload: Option<Message>) -> Result<Self> {
        let content_size = if let Some(ref p) = payload {
            let mut serialized_payload = Vec::new();
            serialize(p, &mut serialized_payload)?;
            serialized_payload.len() as u16
        } else {
            0
        };
        let header = Header::new(content_size);
        Ok(Response {
            header,
            status,
            payload,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        write_to_buffer(
            &self.header,
            self.status(),
            self.payload.as_ref(),
            &mut buffer,
        )?;

        Ok(buffer)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (header, status, payload) = read_from_buffer::<StatusCode>(bytes)?;

        Ok(Response {
            header,
            status,
            payload,
        })
    }

    pub fn status(&self) -> &StatusCode {
        &self.status
    }

    pub fn payload(&self) -> &Option<Message> {
        &self.payload
    }
}

trait CommandOrStatus {
    fn as_u8(&self) -> u8;
}

impl CommandOrStatus for Command {
    fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl CommandOrStatus for StatusCode {
    fn as_u8(&self) -> u8 {
        *self as u8
    }
}

fn write_to_buffer(
    header: &Header,
    command_or_status: &impl CommandOrStatus,
    payload: Option<&Message>,
    buffer: &mut Vec<u8>,
) -> Result<()> {
    header.to_bytes(buffer)?;

    buffer.write_all(&[command_or_status.as_u8()])?;

    if let Some(p) = payload {
        serialize(p, buffer)?;
    }

    Ok(())
}

fn read_from_buffer<T>(bytes: &[u8]) -> Result<(Header, T, Option<Message>)>
where
    T: TryFrom<u8> + Copy,
    T::Error: Into<ProtocolError>,
{
    if bytes.len() < 5 {
        return Err(Error::Protocol(ProtocolError::InvalidMessageFormat));
    }

    let header = Header::from_bytes(&bytes[..4])?;

    let command_or_status = T::try_from(bytes[4]).map_err(|e| Error::Protocol(e.into()))?;

    let payload_bytes = &bytes[5..];

    let payload = if payload_bytes.len() != header.content_size as usize {
        return Err(Error::Protocol(ProtocolError::HeaderMismatch));
    } else if header.content_size > 0 {
        Some(deserialize(payload_bytes)?)
    } else {
        None
    };

    Ok((header, command_or_status, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Result;
    use std::collections::HashMap;

    #[test]
    fn test_request_serialization_deserialization() {
        let message = Message::BlockConfirmation("BlockConfirmed".to_string());
        let request = Request::new(Command::Post, Some(message)).unwrap();

        let serialized = request.to_bytes().unwrap();
        let deserialized = Request::from_bytes(&serialized).unwrap();

        assert_eq!(request.command(), deserialized.command());
        assert_eq!(
            request.payload().is_some(),
            deserialized.payload().is_some()
        );
    }

    #[test]
    fn test_response_serialization_deserialization() {
        let message = Message::PeerIntroduction("NewPeer123".to_string());
        let response = Response::new(StatusCode::OK, Some(message.clone())).unwrap();

        let serialized = response.to_bytes().unwrap();
        let deserialized = Response::from_bytes(&serialized).unwrap();

        assert_ne!(response.status(), &StatusCode::NotFound);
        assert_eq!(response.status(), deserialized.status());
        assert_eq!(
            response.payload().is_some(),
            deserialized.payload().is_some()
        );
        if let Some(Message::PeerIntroduction(ref deserialized_content)) = deserialized.payload() {
            assert_eq!(deserialized_content, "NewPeer123");
        } else {
            panic!("Payload did not match expected PeerIntroduction content");
        }
    }

    #[test]
    fn test_empty_payload_request() -> Result<()> {
        let request = Request::new(Command::Get, None)?;

        let serialized = request.to_bytes()?;
        let deserialized = Request::from_bytes(&serialized)?;

        assert_eq!(request.command(), deserialized.command());
        assert!(deserialized.payload().is_none());
        Ok(())
    }

    #[test]
    fn test_empty_payload_response() -> Result<()> {
        let response = Response::new(StatusCode::NotFound, None)?;

        let serialized = response.to_bytes()?;
        let deserialized = Response::from_bytes(&serialized)?;

        assert_eq!(response.status(), deserialized.status());
        assert!(deserialized.payload().is_none());
        Ok(())
    }
}
