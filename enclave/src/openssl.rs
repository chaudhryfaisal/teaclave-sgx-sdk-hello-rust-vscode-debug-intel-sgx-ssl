use core::slice;

use libc::{c_int, c_uchar};
use sgx_types::sgx_status_t;

pub enum EcKey {}

pub struct Context {
    key: Option<*mut EcKey>,
}

unsafe impl Sync for Context {}

static mut CTX: Context = Context { key: None };

#[no_mangle]
pub unsafe extern "C" fn ecall_openssl_ecc_key_create() -> sgx_status_t {
    if CTX.key.is_none() {
        println!("ecall_openssl_ecc_key_create");
        let eckey = EC_KEY_new_by_curve_name(NID_X9_62_PRIME256V1);
        if eckey.is_null() {
            panic!("Failed to create EcKey object");
        }

        // Generate a new EC key pair
        if EC_KEY_generate_key(eckey) != 1 {
            panic!("Failed to generate EC key pair");
        }
        CTX.key.replace(eckey);
    }
    sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn ecall_openssl_ecc_sign(data: *const u8, data_len: usize) -> sgx_status_t {
    let data = slice::from_raw_parts(data, data_len);
    let sig = ECDSA_do_sign(data.as_ptr(), data.len() as c_int, CTX.key.unwrap());
    if sig.is_null() {
        eprintln!("Failed to sign data");
        panic!("Failed to sign data");
    }
    ECDSA_SIG_free(sig);
    return sgx_status_t::SGX_SUCCESS;
}

const NID_X9_62_PRIME256V1: c_int = 415;

extern "C" {
    fn EC_KEY_new_by_curve_name(nid: c_int) -> *mut EcKey;
    // fn EC_KEY_free(key: *mut EcKey);
    fn EC_KEY_generate_key(key: *mut EcKey) -> c_int;
    // fn EC_KEY_get0_private_key(key: *const EcKey) -> *const EcKey;
    // fn EC_KEY_set_group(key: *mut EcKey, group: *const EcKey) -> c_int;
    // fn EC_KEY_get0_group(key: *const EcKey) -> *const EcKey;
    // fn EC_KEY_set_asn1_flag(key: *mut EcKey, flag: c_long) -> c_int;
    // fn EC_KEY_asn1_get_private_key(key: *const EcKey, priv_key: *mut *mut c_uchar) -> c_int;

    fn ECDSA_do_sign(dgst: *const c_uchar, dgst_len: c_int, eckey: *mut EcKey) -> *mut EcKey;
    // fn ECDSA_do_verify(dgst: *const c_uchar, dgst_len: c_int, sig: *const EcKey, eckey: *mut EcKey) -> c_int;
    fn ECDSA_SIG_free(sig: *mut EcKey);
}
