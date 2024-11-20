use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey};
use rand::{rngs::OsRng, Rng};

use crate::{errors::Result, transaction::Transaction, utxo::UTXO};

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

#[allow(unused)]
pub fn generate_random_utxos(
    sender: [u8; 32],
    input_value: u32,
    output_value: u32,
) -> Result<(Vec<UTXO>, Vec<UTXO>)> {
    let mut rand_gen = rand::thread_rng();

    let mut inputs: Vec<UTXO> = Vec::new();

    let mut input_value = input_value;

    let mut i = 0;
    while input_value > 0 {
        let min_input = input_value % 100;
        let input_val = rand_gen.gen_range(min_input..=input_value);
        i += 1;

        input_value -= input_val;
        let new_utxo = UTXO::new(input_val as u64, i).unwrap();
        // sample transaction hash
        let confirmed_utxo = new_utxo.confirm_utxo(sender, [1u8; 32], 1, i == 0)?;
        inputs.push(confirmed_utxo);
    }

    let mut outputs: Vec<UTXO> = Vec::new();

    let mut output_value = output_value;

    let mut o = 0;
    while output_value > 0 {
        let min_output = output_value % 100;

        let output_val = rand_gen.gen_range(min_output..=output_value);
        o += 1;

        output_value -= output_val;
        outputs.push(UTXO::new(output_val as u64, o).unwrap());
    }

    Ok((inputs, outputs))
}

pub fn create_mock_transaction(value_to_send: u32, value_to_receive: u32) -> (Transaction, String) {
    let (mut signing_key, _, sender, receiver) = generate_key_pairs().unwrap();

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

    (transaction, unlocking_script)
}
