# SGX SSL Performance comparison
This repo is to compare openssl comaprison inside and outside sgx enclave. this example is extension of [hello-rust-vscode-debug](https://github.com/apache/incubator-teaclave-sgx-sdk/tree/master/samplecode/hello-rust-vscode-debug)

# Pre Req
1. Intel SGX SDK
2. Intel SGX SSL
3. openssl-dev
4. Rust / Cargo

# Test
```bash
git clone 
cd teaclave-sgx-sdk-hello-rust-vscode-debug-intel-sgx-ssl
make # test with simulation 
    ...
    cd ./bin; ./app
    !!! stage started: openssl_no_enclave
    *** stage completed: openssl_no_enclave (1_000 iters, 0.023 secs)
    !!! stage started: openssl_enclave
    *** stage completed: openssl_enclave (1_000 iters, 0.026 secs)
    !!! stage started: ring_enclave
    *** stage completed: ring_enclave (1_000 iters, 0.022 secs)
    --- Benchmark results ----------------------------------------
     stage               iters  secs   msecs   iters/s  diff.s
    --------------------------------------------------------------
     openssl_enclave     1_000  0.026  26.258  38_084   -12.79 %
     openssl_no_enclave  1_000  0.023  22.900  43_668
     ring_enclave        1_000  0.022  22.020  45_412   +3.99 %
    lb_openssl_enclave	latency (μs) avg: 26, min: 25, max: 31
    lb_openssl_no_enclave	latency (μs) avg: 22, min: 22, max: 31
    lb_ring_enclave	latency (μs) avg: 22, min: 21, max: 24
    
    cd app-ring; cargo run -q --release
    ring_ecc_key_create
    !!! stage started: ring_no_enclave
    *** stage completed: ring_no_enclave (1_000 iters, 0.023 secs)
    --- Benchmark results ----------------------------------------
     stage            iters  secs   msecs   iters/s  diff.s
    ---------------------------------------------------------
     ring_no_enclave  1_000  0.023  23.362  42_804
    lb_ring_no_enclave	latency (μs) avg: 23, min: 22, max: 26
```

# Test in docker
```bash
git clone
cd teaclave-sgx-sdk-hello-rust-vscode-debug-intel-sgx-ssl
make shell
# inside container shell
./setup-intel-sgx-ssl.sh
make 
```

# simulation
```bash
make bench-sim
    --- Benchmark results ----------------------------------------
     stage               iters  secs   msecs   iters/s  diff.s
    --------------------------------------------------------------
     openssl_enclave     1_000  0.026  26.238  38_112   -12.77 %
     openssl_no_enclave  1_000  0.023  22.887  43_692
     ring_enclave        1_000  0.022  22.059  45_333   +3.76 %
    lb_openssl_enclave	latency (μs) avg: 26, min: 25, max: 34
    lb_openssl_no_enclave	latency (μs) avg: 22, min: 22, max: 26
    lb_ring_enclave	latency (μs) avg: 22, min: 21, max: 25
    
    cd app-ring; cargo run -q --release
    ring_ecc_key_create
    !!! stage started: ring_no_enclave
    *** stage completed: ring_no_enclave (1_000 iters, 0.023 secs)
    --- Benchmark results ----------------------------------------
     stage            iters  secs   msecs   iters/s  diff.s
    ---------------------------------------------------------
     ring_no_enclave  1_000  0.023  23.413  42_711
    lb_ring_no_enclave	latency (μs) avg: 23, min: 22, max: 27
```
# hardware
```bash
make bench-hw
    --- Benchmark results ----------------------------------------
     stage               iters  secs   msecs   iters/s  diff.s
    --------------------------------------------------------------
     openssl_enclave     1_000  0.033  32.742  30_542   -30.17 %
     openssl_no_enclave  1_000  0.023  22.862  43_739
     ring_enclave        1_000  0.026  26.334  37_973   -13.18 %
    lb_openssl_enclave	latency (μs) avg: 32, min: 32, max: 39
    lb_openssl_no_enclave	latency (μs) avg: 23, min: 22, max: 32
    lb_ring_enclave	latency (μs) avg: 26, min: 25, max: 33
    
    cd app-ring; cargo run -q --release
    ring_ecc_key_create
    !!! stage started: ring_no_enclave
    *** stage completed: ring_no_enclave (1_000 iters, 0.023 secs)
    --- Benchmark results ----------------------------------------
     stage            iters  secs   msecs   iters/s  diff.s
    ---------------------------------------------------------
     ring_no_enclave  1_000  0.023  23.416  42_706
    lb_ring_no_enclave	latency (μs) avg: 23, min: 22, max: 27
```