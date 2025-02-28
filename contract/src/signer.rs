use hex::FromHex;
use near_sdk::{ext_contract, near, PromiseOrValue};
use omni_transaction::near::types::Secp256K1Signature;

#[near(serializers = [json])]
pub struct SignRequest {
    pub payload: [u8; 32],
    pub path: String,
    pub key_version: u32,
}

impl SignRequest {
    pub fn new(payload: [u8; 32], path: String, key_version: u32) -> Self {
        Self {
            payload,
            path,
            key_version,
        }
    }
}

#[derive(Debug)]
#[near(serializers = [json])]
pub struct SignResult {
    pub big_r: AffinePoint,
    pub s: Scalar,
    pub recovery_id: u8,
}

impl Into<Secp256K1Signature> for SignResult {
    fn into(self) -> Secp256K1Signature {
        // Get r and s from the sign result
        let big_r = self.big_r.affine_point;
        let s = self.s.scalar;

        // Split big r into its parts
        let r = &big_r[2..];
        let end = &big_r[..2];

        // Convert hex to bytes
        let r_bytes = Vec::from_hex(r).expect("Invalid hex in r");
        let s_bytes = Vec::from_hex(s).expect("Invalid hex in s");
        let end_bytes = Vec::from_hex(end).expect("Invalid hex in end");

        // Add individual bytes together in the correct order
        let mut signature_bytes = [0u8; 65];
        signature_bytes[..32].copy_from_slice(&r_bytes);
        signature_bytes[32..64].copy_from_slice(&s_bytes);
        signature_bytes[64] = end_bytes[0];

        Secp256K1Signature(signature_bytes)
    }
}

#[derive(Debug)]
#[near(serializers = [json])]
pub struct AffinePoint {
    pub affine_point: String,
}

#[derive(Debug)]
#[near(serializers = [json])]
pub struct Scalar {
    pub scalar: String,
}

#[ext_contract(ext_signer)]
pub trait SignerInterface {
    fn sign(&mut self, request: SignRequest) -> PromiseOrValue<SignResult>;
}
