use crate::{CustomPrecompileFn, Error, Precompile, PrecompileAddress, PrecompileResult};

pub const P256VERIFY: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(19),
    Precompile::Custom(p256_verify as CustomPrecompileFn),
);

fn p256_verify(i: &[u8], target_gas: u64) -> PrecompileResult {
    use core::cmp::min;
    use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};

    const P256VERIFY_BASE: u64 = 3_450;

    if P256VERIFY_BASE > target_gas {
        return Err(Error::OutOfGas);
    }
    let mut input = [0u8; 160];
    input[..min(i.len(), 160)].copy_from_slice(&i[..min(i.len(), 160)]);

    let mut msg = [0u8; 32];
    let mut sig = [0u8; 64];
    let mut pk = [0u8; 64];
    let mut uncompressed_pk = [0u8; 65];

    msg[0..32].copy_from_slice(&input[0..32]);
    // r: signature
    sig[0..32].copy_from_slice(&input[32..64]);
    // s: signature
    sig[32..64].copy_from_slice(&input[64..96]);
    // x: public key
    pk[0..32].copy_from_slice(&input[96..128]);
    // y: public key
    pk[32..64].copy_from_slice(&input[128..160]);
    // append 0x04 to the public key: uncompressed form
    uncompressed_pk[0] = 0x04;
    uncompressed_pk[1..].copy_from_slice(&pk);

    let signature: Signature = Signature::from_slice(&sig).unwrap();
    let public_key: VerifyingKey = VerifyingKey::from_sec1_bytes(&uncompressed_pk).unwrap();

    let mut result = [0u8; 32];

    // verify
    if public_key.verify(&msg, &signature).is_ok() {
        result[31] = 0x01;
        Ok((P256VERIFY_BASE, result.into()))
    } else {
        Ok((P256VERIFY_BASE, result.into()))
    }
}

#[cfg(test)]
mod test {
    use revm_primitives::hex_literal::hex;

    use super::p256_verify;
    #[test]
    fn proper_sig_verify() {
        let input = hex!("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e");
        let target_gas = 3_500u64;
        let (gas_used, res) = p256_verify(&input, target_gas).unwrap();
        assert_eq!(gas_used, 3_450u64);
        let mut expected_res = [0u8; 32];
        expected_res[31] = 1;
        assert_eq!(res, expected_res.to_vec());
    }

    #[test]
    fn verify_ok() {
        use p256::{ecdsa::{SigningKey, Signature, signature::Signer}};

        // Signing
        let signing_key = SigningKey::from_slice(&hex!("45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8")).unwrap(); // Serialize with `::to_bytes()`
        let message = b"ECDSA proves knowledge of a secr";
        let signature: Signature = signing_key.sign(message);

        // Verification
        use p256::ecdsa::{VerifyingKey, signature::Verifier};
        let verifying_key = VerifyingKey::from(&signing_key); // Serialize with `::to_encoded_point()`
        assert!(verifying_key.verify(message, &signature).is_ok());

        let target_gas = 3_500u64;
        let mut input = [0u8; 160];
        input[..32].copy_from_slice(message);
        input[32..96].copy_from_slice(&signature.to_bytes());
        input[96..].copy_from_slice(&verifying_key.to_sec1_bytes()[1..]);

        let (gas_used, res) = p256_verify(&input, target_gas).unwrap();
        assert_eq!(gas_used, 3_450u64);
        let mut expected_res = [0u8; 32];
        expected_res[31] = 1;
        assert_eq!(res, expected_res.to_vec());
    }
}