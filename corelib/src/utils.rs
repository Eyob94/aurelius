use crate::errors::{Error, Result};

pub fn convert_u8_to_u832(raw: &[u8]) -> Result<&[u8; 32]> {
    if raw.len() != 32 {
        Err(Error::InvalidU8Length(raw.len()))
    } else {
        // SAFETY: We checked the length, so this is safe
        Ok(unsafe { &*(raw.as_ptr() as *const [u8; 32]) })
    }
}


pub fn convert_u8_to_u864(raw: &[u8]) -> Result<&[u8; 64]> {
    if raw.len() != 64 {
        Err(Error::InvalidU8Length(raw.len()))
    } else {
        // SAFETY: We checked the length, so this is safe
        Ok(unsafe { &*(raw.as_ptr() as *const [u8; 64]) })
    }
}



