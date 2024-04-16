use ring::{rand, signature};
use ring::signature::EcdsaKeyPair;

struct Context {
    key: Option<EcdsaKeyPair>,
}

static mut CTX: Context = Context { key: None };

pub unsafe fn ring_ecc_key_create() {
    if CTX.key.is_none() {
        println!("ring_ecc_key_create");
        let rng = rand::SystemRandom::new();
        let alg = &signature::ECDSA_P256_SHA256_ASN1_SIGNING;
        let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(alg, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(alg, pkcs8_bytes.as_ref(), &rng).unwrap();
        CTX.key.replace(key_pair);
    }
}

pub unsafe fn ring_ecc_sign(data: &[u8]) {
    let rng = rand::SystemRandom::new();
    let key_pair = CTX.key.as_ref().unwrap();
    key_pair.sign(&rng, data).unwrap();
}


pub fn ring_ecc_sign_test() {
    let data = b"hello world";
    unsafe {
        ring_ecc_key_create();
        ring_ecc_sign(data);
    }
}

#[test]
fn ring_ecc_test() {
    ring_ecc_sign_test();
}
