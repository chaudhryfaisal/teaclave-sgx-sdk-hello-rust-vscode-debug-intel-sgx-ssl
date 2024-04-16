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

use bma_benchmark::LatencyBenchmark;
use openssl::{openssl_ecc_sign, openssl_ecc_sign_test};

mod openssl;

static ENCLAVE_FILE: &'static str = "enclave.signed.so";

extern {
    fn say_something(eid: sgx_enclave_id_t, retval: *mut sgx_status_t, some_string: *const u8, len: usize) -> sgx_status_t;
    fn ecall_openssl_ecc_key_create(eid: sgx_enclave_id_t) -> sgx_status_t;
    fn ecall_openssl_ecc_sign(eid: sgx_enclave_id_t, clear_text: *const u8, clear_text_len: usize) -> sgx_status_t;
    fn ecall_ring_ecc_key_create(eid: sgx_enclave_id_t) -> sgx_status_t;
    fn ecall_ring_ecc_sign(eid: sgx_enclave_id_t, clear_text: *const u8, clear_text_len: usize) -> sgx_status_t;
    fn ecall_empty_function(eid: sgx_enclave_id_t, clear_text: *const u8, clear_text_len: usize) -> sgx_status_t;
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
    unsafe { benchmark(enclave.geteid()); }
    enclave.destroy();
}

unsafe fn benchmark(eid: sgx_enclave_id_t) {
    let n = 1_000;
    let data = b"hello world";

    // warmup
    openssl_ecc_sign_test();
    openssl_ecc_enclave_test(eid);
    ring_ecc_enclave_test(eid);
    empty_function(data.as_ptr(), data.len());


    //empty_function_no_enclave
    staged_benchmark_start!("empty_function_no_enclave");
    for _ in 0..n {
        empty_function(data.as_ptr(), data.len());
    }
    staged_benchmark_finish_current!(n);
    let mut lb_empty_no_enclave = LatencyBenchmark::new();
    for _ in 0..n {
        lb_empty_no_enclave.op_start();
        empty_function(data.as_ptr(), data.len());
        lb_empty_no_enclave.op_finish();
    }


    //empty_function_no_enclave
    staged_benchmark_start!("empty_function_enclave");
    for _ in 0..n {
        ecall_empty_function(eid, data.as_ptr(), data.len());
    }
    staged_benchmark_finish_current!(n);

    let mut lb_empty_enclave = LatencyBenchmark::new();
    for _ in 0..n {
        lb_empty_enclave.op_start();
        ecall_empty_function(eid, data.as_ptr(), data.len());
        lb_empty_enclave.op_finish();
    }


    //openssl_no_enclave
    staged_benchmark_start!("openssl_no_enclave");
    for _ in 0..n {
        openssl_ecc_sign(data);
    }
    staged_benchmark_finish_current!(n);

    let mut lb_openssl_no_enclave = LatencyBenchmark::new();
    for _ in 0..n {
        lb_openssl_no_enclave.op_start();
        openssl_ecc_sign(data);
        lb_openssl_no_enclave.op_finish();
    }

    //openssl_enclave
    staged_benchmark_start!("openssl_enclave");
    for _ in 0..n {
        let result = ecall_openssl_ecc_sign(eid, data.as_ptr() as *const u8, data.len());
        assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    }
    staged_benchmark_finish_current!(n);

    let mut lb_openssl_enclave = LatencyBenchmark::new();
    for _ in 0..n {
        lb_openssl_enclave.op_start();
        ecall_openssl_ecc_sign(eid, data.as_ptr() as *const u8, data.len());
        lb_openssl_enclave.op_finish();
    }

    //ring_enclave
    staged_benchmark_start!("ring_enclave");
    for _ in 0..n {
        let result = ecall_ring_ecc_sign(eid, data.as_ptr() as *const u8, data.len());
        assert_eq!(result, sgx_status_t::SGX_SUCCESS);
    }
    staged_benchmark_finish_current!(n);

    let mut lb_ring_enclave = LatencyBenchmark::new();
    for _ in 0..n {
        lb_ring_enclave.op_start();
        ecall_ring_ecc_sign(eid, data.as_ptr() as *const u8, data.len());
        lb_ring_enclave.op_finish();
    }

    //print_results
    staged_benchmark_print_for!("openssl_no_enclave");
    print!("lb_empty_enclave\t");
    lb_empty_enclave.print();
    print!("lb_empty_no_enclave\t");
    lb_empty_no_enclave.print();
    print!("lb_openssl_enclave\t");
    lb_openssl_enclave.print();
    print!("lb_openssl_no_enclave\t");
    lb_openssl_no_enclave.print();
    print!("lb_ring_enclave\t");
    lb_ring_enclave.print();
}

fn empty_function(_data: *const u8, _len: usize) {

}

unsafe fn openssl_ecc_enclave_test(eid: sgx_enclave_id_t) {
    println!("[+] openssl_ecc_enclave_test");
    println!("[+] ecall_openssl_ecc_key_create");
    let result = ecall_openssl_ecc_key_create(eid);
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);

    println!("[+] ecall_openssl_ecc_sign");
    let data = String::from("to sign");
    let result = ecall_openssl_ecc_sign(eid, data.as_ptr() as *const u8, data.len());
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
}

unsafe fn ring_ecc_enclave_test(eid: sgx_enclave_id_t) {
    println!("[+] ring_ecc_enclave_test");
    println!("[+] ecall_ring_ecc_key_create");
    let result = ecall_ring_ecc_key_create(eid);
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);

    println!("[+] ecall_ring_ecc_sign");
    let data = String::from("to sign");
    let result = ecall_ring_ecc_sign(eid, data.as_ptr() as *const u8, data.len());
    assert_eq!(result, sgx_status_t::SGX_SUCCESS);
}
