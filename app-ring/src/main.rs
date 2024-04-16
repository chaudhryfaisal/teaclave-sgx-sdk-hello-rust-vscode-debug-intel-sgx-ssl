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
extern crate ring;

use bma_benchmark::LatencyBenchmark;

use crate::ring_ecc::{ring_ecc_sign, ring_ecc_sign_test};

mod ring_ecc;


fn main() {
    unsafe { benchmark(); }
}

unsafe fn benchmark() {
    let n = 1_000;
    let data = b"hello world";

    // warmup
    ring_ecc_sign_test();

    //ring_no_enclave
    staged_benchmark_start!("ring_no_enclave");
    for _ in 0..n {
        ring_ecc_sign(data);
    }
    staged_benchmark_finish_current!(n);

    let mut lb_ring_no_enclave = LatencyBenchmark::new();
    for _ in 0..n {
        lb_ring_no_enclave.op_start();
        ring_ecc_sign(data);
        lb_ring_no_enclave.op_finish();
    }

    //print_results
    staged_benchmark_print_for!("ring_no_enclave");
    print!("lb_ring_no_enclave\t");
    lb_ring_no_enclave.print();
}