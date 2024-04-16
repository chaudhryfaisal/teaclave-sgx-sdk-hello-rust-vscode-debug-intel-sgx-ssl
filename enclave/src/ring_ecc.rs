use core::slice;

use ring::{rand, signature};
use ring::signature::EcdsaKeyPair;
use sgx_types::sgx_status_t;

struct Context {
    key: Option<EcdsaKeyPair>,
}

static mut CTX: Context = Context { key: None };

#[no_mangle]
pub unsafe fn ecall_ring_ecc_key_create() -> sgx_status_t {
    if CTX.key.is_none() {
        println!("ecall_ring_ecc_key_create");
        let rng = rand::SystemRandom::new();
        let alg = &signature::ECDSA_P256_SHA256_ASN1_SIGNING;
        let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(alg, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(alg, pkcs8_bytes.as_ref(), &rng).unwrap();
        CTX.key.replace(key_pair);
    }
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ecall_ring_ecc_sign(data: *const u8, data_len: usize) -> sgx_status_t {
    let data = slice::from_raw_parts(data, data_len);
    let rng = rand::SystemRandom::new();
    let key_pair = CTX.key.as_ref().unwrap();
    key_pair.sign(&rng, data).unwrap();
    sgx_status_t::SGX_SUCCESS
}
