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
