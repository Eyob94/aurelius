use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

use crate::errors::Result;

#[allow(unused)]
pub fn generate_key_pairs() -> Result<(SigningKey, SigningKey, [u8; 32], [u8; 32])> {
    let mut csprng = OsRng;

    let signing_key = SigningKey::generate(&mut csprng);
    let mut csprng2 = OsRng;
    let receiver_singing_key = SigningKey::generate(&mut csprng2);

    let sender = signing_key.verifying_key().to_bytes();
    let receiver = receiver_singing_key.verifying_key().to_bytes();

    assert_ne!(sender, receiver);

    Ok((signing_key, receiver_singing_key, sender, receiver))
}
