use borsh::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::{ed25519::signature::SignerMut, Signature, SigningKey, VerifyingKey};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    errors::{Error, Result},
    utxo::UTXO,
};

#[allow(unused)]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct Transaction {
    pub hash_id: [u8; 32],
    pub sender: [u8; 32],
    pub receiver: [u8; 32],
    pub timestamp: u128,
    pub signature: [u8; 64],
    // For newly minted coins there will be no inputs
    pub inputs: Vec<UTXO>,
    pub outputs: Vec<UTXO>,
}

impl Transaction {
    pub fn new(signing_key: &mut SigningKey, receiver: [u8; 32]) -> Result<Self> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let sender = signing_key.verifying_key().to_bytes();

        let mut txn = Self {
            hash_id: [0u8; 32],
            sender,
            receiver,
            timestamp,
            signature: [0u8; 64],
            inputs: vec![],
            outputs: vec![],
        };

        txn.calculate_hash(signing_key);

        Ok(txn)
    }

    fn calculate_hash(&mut self, signing_key: &mut SigningKey) {
        let mut serialized = Vec::new();

        serialized.extend(&self.sender);
        serialized.extend(&self.receiver);
        serialized.extend(&self.timestamp.to_le_bytes());

        for input in self.inputs.iter() {
            serialized.extend(input.to_bytes())
        }

        for output in self.outputs.iter() {
            serialized.extend(output.to_bytes())
        }
        self.hash_id = *blake3::hash(serialized.as_slice()).as_bytes();
        self.signature = signing_key.sign(&self.hash_id).to_bytes();
    }

    pub fn add_inputs(
        &mut self,
        new_inputs: Vec<UTXO>,
        signing_key: &mut SigningKey,
    ) -> Result<()> {
        if new_inputs.iter().any(|u| matches!(u, UTXO::Pending { .. })) {
            return Err(Error::PendingUTXO);
        }

        if new_inputs.is_empty() {
            return Err(Error::InsufficientFunds);
        }

        self.inputs.extend_from_slice(new_inputs.as_slice());

        self.calculate_hash(signing_key);

        Ok(())
    }

    pub fn add_outputs(
        &mut self,
        new_outputs: Vec<UTXO>,
        signing_key: &mut SigningKey,
    ) -> Result<()> {
        if new_outputs
            .iter()
            .any(|u| matches!(u, UTXO::Confirmed { .. }))
        {
            return Err(Error::ConfirmedUTXO);
        }
        if new_outputs.is_empty() {
            return Err(Error::InsufficientFunds);
        }

        self.outputs.extend_from_slice(new_outputs.as_slice());

        self.calculate_hash(signing_key);

        Ok(())
    }

    // This verifies the sender holds sufficient funds to carry out the
    // transaction.
    // It also checks that the transaction was initiated by the rightful owner as well
    // as the ownership of the inputs are also verified
    pub fn verify(&self, unlocking_script: &str) -> Result<(u64, u64, u64)> {
        let pub_key = VerifyingKey::from_bytes(&self.sender)?;

        // Check if any inputs are unfonfirmed yet, and sum them
        let input: u64 = self
            .inputs
            .iter()
            .map(|utxo| match utxo {
                UTXO::Confirmed { value, .. } => Ok(*value),
                UTXO::Pending { .. } => Err(Error::PendingUTXO),
            })
            .collect::<Result<Vec<u64>>>()?
            .iter()
            .sum();

        // Check if any outputs are confirmed already, and sum them
        let output: u64 = self
            .outputs
            .iter()
            .map(|utxo| match utxo {
                UTXO::Pending { value, .. } => Ok(*value),
                UTXO::Confirmed { .. } => Err(Error::ConfirmedUTXO),
            })
            .collect::<Result<Vec<u64>>>()?
            .iter()
            .sum();

        if output > input {
            return Err(Error::InsufficientFunds);
        }

        // Subtract only if input >= output to prevent subtraction overflow
        let fee = input - output;

        // Unlock the utxo using the unlocking script
        for utxo in self.inputs.iter() {
            utxo.unlock(unlocking_script)?;
        }

        let signature: Signature = Signature::from_bytes(&self.signature);

        pub_key
            .verify_strict(&self.hash_id, &signature)
            .map_err(|_| Error::UnAuthorized)?;

        Ok((input, output, fee))
    }
}

#[cfg(test)]
mod test {

    use ed25519_dalek::ed25519::signature::SignerMut;

    use crate::{
        errors::Error,
        test_utils::{generate_key_pairs, generate_random_utxos},
    };

    use super::Transaction;

    #[test]
    fn create_and_verify_txn() {
        let (mut signing_key, _, sender, receiver) = generate_key_pairs().unwrap();

        let value_to_send = 1_000_000_000_u32;
        let value_to_receive = value_to_send - 10;

        let mut transaction = Transaction::new(&mut signing_key, receiver).unwrap();

        // Receive 999...million
        let (input_utxo, output_utxo) =
            generate_random_utxos(sender, value_to_send, value_to_receive).unwrap();

        transaction
            .add_outputs(output_utxo, &mut signing_key)
            .unwrap();
        transaction
            .add_inputs(input_utxo, &mut signing_key)
            .unwrap();

        let sender_hash = blake3::hash(&sender);
        let signature = signing_key.sign(sender_hash.as_bytes()).to_bytes();

        let unlocking_script = format!("{} {}", hex::encode(signature), hex::encode(sender));

        let (_, _, fee) = transaction.verify(&unlocking_script).unwrap();

        assert_eq!(fee, 10)
    }

    #[test]
    fn fails_on_insufficient_funds() {
        let (mut signing_key, _, sender, receiver) = generate_key_pairs().unwrap();

        let value_to_send = 1_000_000_000_u32;
        let value_to_receive = value_to_send + 10;

        let mut transaction = Transaction::new(&mut signing_key, receiver).unwrap();

        let (input_utxo, output_utxo) =
            generate_random_utxos(sender, value_to_send, value_to_receive).unwrap();

        transaction
            .add_inputs(input_utxo, &mut signing_key)
            .unwrap();
        transaction
            .add_outputs(output_utxo, &mut signing_key)
            .unwrap();

        let sender_hash = blake3::hash(&sender);
        let signature = signing_key.sign(sender_hash.as_bytes()).to_bytes();

        let unlocking_script = format!("{} {}", hex::encode(signature), hex::encode(sender));

        assert!(matches!(
            transaction.verify(&unlocking_script),
            Err(Error::InsufficientFunds)
        ));
    }

    #[test]
    fn fails_on_wrong_sender() {
        let (mut s, mut signing_key, sender, receiver) = generate_key_pairs().unwrap();

        let value_to_send = 1_000_000_000_u32;
        let value_to_receive = value_to_send - 10;

        let mut transaction = Transaction::new(&mut signing_key, receiver).unwrap();

        let (input_utxo, output_utxo) =
            generate_random_utxos(sender, value_to_send, value_to_receive).unwrap();

        transaction.add_inputs(input_utxo, &mut s).unwrap();
        transaction.add_outputs(output_utxo, &mut s).unwrap();

        let sender_hash = blake3::hash(&sender);
        let signature = s.sign(sender_hash.as_bytes()).to_bytes();

        let unlocking_script = format!("{} {}", hex::encode(signature), hex::encode(sender));

        assert!(matches!(
            transaction.verify(&unlocking_script),
            Err(Error::UnAuthorized)
        ))
    }
}
