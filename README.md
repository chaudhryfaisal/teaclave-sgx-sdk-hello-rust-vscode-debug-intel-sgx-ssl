# SGX SSL Performance comparison
This repo is to compare openssl comaprison inside and outside sgx enclave. this example is extension of [hello-rust-vscode-debug](https://github.com/apache/incubator-teaclave-sgx-sdk/tree/master/samplecode/hello-rust-vscode-debug)

# Pre Req
1. Intel SGX SDK
2. Intel SGX SSL
3. openssl-dev
4. Rust / Cargo

# Test
- Note: `if make command fails, try to run again :-)` 
```bash
git clone 
cd hello-rust-vscode-debug
make
    ...
    cd ./bin; ./app
    [+] Init Enclave Successful 2!
    This is a normal world string passed into Enclave!
    This is a in-Enclave Rust string!
    [enclave/src/lib.rs:66] a * 2 = 4
    [+] say_something success...
    [+] ecc_test
    [+] ecall_ecc_key_create
    ecall_ecc_key_create
    [+] ecall_ecc_sign
    [+] ecc_test
    [+] ecall_ecc_key_create
    ecall_ecc_key_create
    [+] ecall_ecc_sign
    ecc_key_create
    !!! stage started: openssl_no_enclave 
    *** stage completed: openssl_no_enclave (1_000 iters, 0.023 secs)
    !!! stage started: openssl_enclave 
    *** stage completed: openssl_enclave (1_000 iters, 0.198 secs)
    --- Benchmark results ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
     stage               iters  secs   msecs    iters/s  diff.s     
    ----------------------------------------------------------------
     openssl_enclave     1_000  0.198  197.980  5_051               
     openssl_no_enclave  1_000  0.023  22.962   43_549   +762.19 %  

```