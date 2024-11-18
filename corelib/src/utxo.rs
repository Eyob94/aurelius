use std::time::{SystemTime, UNIX_EPOCH};

use borsh::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::{Signature, VerifyingKey};

use crate::{
    errors::{Error, Result},
    utils::{convert_u8_to_u832, convert_u8_to_u864},
};

#[allow(clippy::style)]
#[derive(Debug, Clone, Hash, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum UTXO {
    Pending {
        // hash used to identify UTXO
        // The value of the UTXO, must be non zero
        value: u64,
        // Index of the utxo in the transaction
        index: u32,
    },
    Confirmed {
        id: [u8; 32],
        script_pubkey: String,
        value: u64,
        txn_hash: [u8; 32],
        index: u32,
        // Timestamp of the block the UTXO was created
        created_at: u32,
        // Height of the block the UTXO was included in
        block_height: u32,
        // Coin earned from mining
        is_coinbase: bool,
    },
}

impl UTXO {
    pub fn new(value: u64, index: u32) -> Result<Self> {
        if value == 0 {
            return Err(Error::InvalidUTXOValue);
        }

        Ok(Self::Pending { value, index })
    }

    pub fn confirm_utxo(
        self,
        owner: [u8; 32],
        txn_hash: [u8; 32],
        block_height: u32,
        coinbase: bool,
    ) -> Result<UTXO> {
        match self {
            UTXO::Pending { value, index } => {
                let mut id = [0u8; 32];
                let id_hash = blake3::hash(&[txn_hash.as_ref(), &index.to_le_bytes()].concat());
                id.copy_from_slice(id_hash.as_bytes());

                let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u32;

                let owner_hash = blake3::hash(&owner);

                Ok(UTXO::Confirmed {
                    id,
                    script_pubkey: format!("{} OP_CHECKSIG", owner_hash),
                    value,
                    txn_hash,
                    index,
                    created_at,
                    block_height,
                    is_coinbase: coinbase,
                })
            }
            UTXO::Confirmed { .. } => Err(Error::ConfirmedUTXO),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            UTXO::Confirmed {
                id,
                script_pubkey,
                value,
                index,
                created_at,
                block_height,
                ..
            } => {
                let mut bytes = Vec::new();
                bytes.extend(id); //32 bytes
                bytes.extend(script_pubkey.as_bytes());
                bytes.extend(&value.to_le_bytes()); // 8 bytes
                bytes.extend(&index.to_le_bytes()); // 4 bytes
                bytes.extend(&created_at.to_le_bytes()); // 4 bytes
                bytes.extend(&block_height.to_le_bytes()); // 4 bytes

                bytes
            }

            UTXO::Pending { value, index, .. } => {
                let mut bytes = Vec::new();
                bytes.extend(&value.to_le_bytes()); // 8 bytes
                bytes.extend(&index.to_le_bytes()); // 4 bytes
                                                    //
                bytes
            }
        }
    }

    pub fn unlock(&self, unlocking_script: &str) -> Result<()> {
        match self {
            UTXO::Pending { .. } => Err(Error::PendingUTXO),
            UTXO::Confirmed { script_pubkey, .. } => {
                let mut stack = Vec::new();

                for token in unlocking_script.split_whitespace() {
                    stack.push(token);
                }

                for token in script_pubkey.split_whitespace() {
                    match token {
                        // Duplicate the top value on the stack
                        "OP_CHECKSIG" => {
                            if stack.len() < 3 {
                                return Err(Error::InvalidUnlockingScript);
                            }

                            let public_key_hash = stack.pop().ok_or_else(|| Error::EmptyStack)?;
                            let public_key =
                                hex::decode(stack.pop().ok_or_else(|| Error::EmptyStack)?)?;
                            let signature =
                                hex::decode(stack.pop().ok_or_else(|| Error::EmptyStack)?)?;
                            let new_hash = blake3::hash(public_key.as_slice());

                            if public_key_hash != new_hash.to_string() {
                                return Err(Error::InvalidUnlockingScript);
                            }
                            if verify_signature(
                                public_key.as_slice(),
                                signature.as_slice(),
                                new_hash.as_bytes(),
                            )
                            .is_err()
                            {
                                return Err(Error::InvalidUnlockingScript);
                            }

                            stack.push("true");
                        }

                        _ => stack.push(token),
                    }
                }

                if stack.len() == 1 && stack.pop().ok_or_else(|| Error::EmptyStack)? == "true" {
                    Ok(())
                } else {
                    Err(Error::InvalidUnlockingScript)
                }
            }
        }
    }
}

fn verify_signature(public_key: &[u8], signature: &[u8], txn_hash: &[u8]) -> Result<()> {
    let verifier = VerifyingKey::from_bytes(convert_u8_to_u832(public_key)?)?;

    let signature = Signature::from_bytes(convert_u8_to_u864(signature)?);

    Ok(verifier.verify_strict(txn_hash, &signature)?)
}

#[cfg(test)]
mod test {
    use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey};
    use rand::rngs::OsRng;

    use super::*;

    #[test]
    fn test_valid_utxo_lifecycle() {
        let mut csprng = OsRng;
        let mut signing_key = SigningKey::generate(&mut csprng);

        let owner = signing_key.verifying_key().to_bytes();
        let txn_hash = [1u8; 32];
        let pending_utxo = UTXO::new(1000, 1).expect("Failed to create UTXO");

        let confirmed_utxo = pending_utxo
            .confirm_utxo(owner, txn_hash, 100, false)
            .expect("Failed to confirm UTXO");

        if let UTXO::Confirmed {
            value,
            block_height,
            is_coinbase,
            ..
        } = confirmed_utxo
        {
            assert_eq!(value, 1000);
            assert_eq!(block_height, 100);
            assert!(!is_coinbase);

            let owner_hash = blake3::hash(&owner);

            let signature = signing_key.sign(owner_hash.as_bytes()).to_bytes();

            let unlocking_script = format!("{} {}", hex::encode(signature), hex::encode(owner));

            confirmed_utxo.unlock(&unlocking_script).unwrap();
        } else {
            panic!("Expected a Confirmed UTXO");
        }
    }
}
