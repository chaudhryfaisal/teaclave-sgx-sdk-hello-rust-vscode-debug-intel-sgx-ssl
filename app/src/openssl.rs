use std::ffi::{c_int, c_uchar};

enum EcKey {}

struct Context {
    key: Option<*mut EcKey>,
}


static mut CTX: Context = Context { key: None };

#[no_mangle]
pub unsafe fn openssl_ecc_key_create() {
    println!("openssl_ecc_key_create");
    let eckey = EC_KEY_new_by_curve_name(415);
    if eckey.is_null() {
        panic!("Failed to create EcKey object");
    }

    // Generate a new EC key pair
    if EC_KEY_generate_key(eckey) != 1 {
        panic!("Failed to generate EC key pair");
    }
    CTX.key.replace(eckey);
}

pub unsafe fn openssl_ecc_sign(data: &[u8]) {
    let sig = ECDSA_do_sign(data.as_ptr(), data.len() as sgx_types::c_int, CTX.key.unwrap());
    if sig.is_null() {
        panic!("Failed to sign data");
    }
    ECDSA_SIG_free(sig);
}

pub fn openssl_ecc_sign_test() {
    let data = b"hello world";
    unsafe {
        openssl_ecc_key_create();
        openssl_ecc_sign(data);
    }
}

#[test]
fn openssl_sys_openssl_ecc_test_no_dep_mod() {
    openssl_ecc_sign_test();
}

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
