// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

#[macro_use]
extern crate bma_benchmark;
extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

use openssl::{ecc_key_create, ecc_sign};

mod openssl;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    fn say_something(eid: sgx_enclave_id_t, retval: *mut sgx_status_t,
                     some_string: *const u8, len: usize) -> sgx_status_t;
    fn ecall_ecc_key_create(eid: sgx_enclave_id_t, retval: *mut sgx_status_t) -> sgx_status_t;
    fn ecall_ecc_sign(eid: sgx_enclave_id_t, retval: *mut sgx_status_t, clear_text: *const u8, clear_text_len: usize) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t { secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 }, misc_select: 0 };
    SgxEnclave::create(ENCLAVE_FILE,
                       debug,
                       &mut launch_token,
                       &mut launch_token_updated,
                       &mut misc_attr)
}

fn main() {
    let enclave = match init_enclave() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };

    let input_string = String::from("This is a normal world string passed into Enclave!\n");

    let mut retval = sgx_status_t::SGX_SUCCESS;

    let result = unsafe {
        say_something(enclave.geteid(),
                      &mut retval,
                      input_string.as_ptr() as *const u8,
                      input_string.len())
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return;
        }
    }

    println!("[+] say_something success...");
    unsafe { ecc_test(enclave.geteid()); }
    unsafe { benchmark(enclave.geteid()); }
    enclave.destroy();
}

unsafe fn benchmark(eid: sgx_enclave_id_t) {
    ecc_test(eid); // warmup
    let n = 1_000;
    let data = b"hello world";


    ecc_key_create();
    staged_benchmark_start!("openssl_no_enclave");
    for _ in 0..n {
        ecc_sign(data);
    }
    staged_benchmark_finish_current!(n);

    let mut retval = sgx_status_t::SGX_SUCCESS;
    staged_benchmark_start!("openssl_enclave");
    for _ in 0..n {
        let result = ecall_ecc_sign(eid, &mut retval, data.as_ptr() as *const u8, data.len());
        assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    }
    staged_benchmark_finish_current!(n);
    staged_benchmark_print_for!("openssl_enclave");
}

unsafe fn ecc_test(eid: sgx_enclave_id_t) {
    println!("[+] ecc_test");
    let mut retval = sgx_status_t::SGX_SUCCESS;
    println!("[+] ecall_ecc_key_create");
    let result = ecall_ecc_key_create(eid, &mut retval);
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);

    println!("[+] ecall_ecc_sign");
    let data = String::from("to sign");
    let result = ecall_ecc_sign(eid, &mut retval, data.as_ptr() as *const u8, data.len());
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
}
