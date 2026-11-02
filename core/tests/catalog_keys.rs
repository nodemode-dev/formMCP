use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use formmcp_core::catalog::{keys, verify};

const PRODUCTION: &str = "P3Cgp37EmB2jOBtTWRNFtO8xMq8hsb+Ii3U5+UrYNtY=";

fn encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn test_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

#[test]
fn the_production_catalogue_key_is_compiled_in_and_well_formed() {
    assert!(keys().iter().any(|k| k == PRODUCTION), "{:?}", keys());
    let raw = base64::engine::general_purpose::STANDARD.decode(PRODUCTION).unwrap();
    let array = <[u8; 32]>::try_from(raw.as_slice()).expect("32 bytes");
    VerifyingKey::from_bytes(&array).expect("a valid Ed25519 public key");
}

#[test]
fn the_compiled_in_keys_verify_beside_a_test_key_and_refuse_a_stranger() {
    let body = br#"{"format":"formai-catalog","products":[]}"#;
    let key = test_key(31);
    let signature = encode(&key.sign(body).to_bytes());
    let mut trusted = keys();
    trusted.push(encode(key.verifying_key().as_bytes()));
    verify(body, &signature, &trusted).unwrap();
    assert!(verify(body, &signature, &[PRODUCTION.to_string()]).is_err(), "the production key accepts only its own signatures");
    let mut tampered = body.to_vec();
    tampered[3] ^= 0x01;
    assert!(verify(&tampered, &signature, &trusted).is_err());
}
