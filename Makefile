# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

######## SGX SDK Settings ########

SGX_SDK ?= /opt/intel/sgxsdk
SGX_MODE ?= SIM
SGX_ARCH ?= x64
SGX_SSL ?= ${shell realpath ${SGX_SDK}/../sgxssl}
SGX_SSL_LIBRARY_PATH ?= ${SGX_SSL}/lib64
SUP_DIR = supporting
include $(SUP_DIR)/buildenv.mk
export SGX_MODE
ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH = x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH = x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS = -m32
	SGX_LIBRARY_PATH = $(SGX_SDK)/lib
	SGX_ENCLAVE_SIGNER = $(SGX_SDK)/bin/x86/sgx_sign
	SGX_EDGER8R = $(SGX_SDK)/bin/x86/sgx_edger8r
else
	SGX_COMMON_CFLAGS = -m64
	SGX_LIBRARY_PATH = $(SGX_SDK)/lib64
	SGX_ENCLAVE_SIGNER = $(SGX_SDK)/bin/x64/sgx_sign
	SGX_EDGER8R = $(SGX_SDK)/bin/x64/sgx_edger8r
endif

ifeq ($(SGX_DEBUG), 1)
ifeq ($(SGX_PRERELEASE), 1)
$(error Cannot set SGX_DEBUG and SGX_PRERELEASE at the same time!!)
endif
endif

ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g -ggdb
	App_Rust_Flags =
	App_Rust_Path = target/debug
else
	SGX_COMMON_CFLAGS += -O2
	App_Rust_Flags += --release
	App_Rust_Path = target/release
endif

SGX_COMMON_CFLAGS += -fstack-protector

######## CUSTOM Settings ########

CUSTOM_LIBRARY_PATH = ${PWD}/lib
CUSTOM_BIN_PATH = ./bin
CUSTOM_EDL_PATH = ${SUP_DIR}/edl
CUSTOM_COMMON_PATH = ${SUP_DIR}/common

######## EDL Settings ########

Enclave_EDL_Files = enclave/Enclave_t.c enclave/Enclave_t.h app/Enclave_u.c app/Enclave_u.h

######## APP Settings ########

App_SRC_Files = $(shell find app/ -type f -name '*.rs') $(shell find app/ -type f -name 'Cargo.toml')
App_Include_Paths = -I ./app -I./include -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH)
App_C_Flags = $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes $(App_Include_Paths)

App_Enclave_u_Object =app/libEnclave_u.a
App_Name = ${CUSTOM_BIN_PATH}/app

######## Enclave Settings ########
ifneq ($(SGX_MODE), HW)
	Trts_Library_Name = sgx_trts_sim
	Service_Library_Name = sgx_tservice_sim
else
	Trts_Library_Name = sgx_trts
	Service_Library_Name = sgx_tservice
endif
Crypto_Library_Name = sgx_tcrypto
KeyExchange_Library_Name = sgx_tkey_exchange
ProtectedFs_Library_Name = sgx_tprotected_fs

RustEnclave_C_Files = $(wildcard ./enclave/*.c)
RustEnclave_C_Objects = $(RustEnclave_C_Files:.c=.o)
RustEnclave_Include_Paths = -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(SGX_SDK)/include -I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid -I ./enclave -I./include

RustEnclave_Link_Libs = -L$(CUSTOM_LIBRARY_PATH) -lenclave
RustEnclave_Compile_Flags = $(SGX_COMMON_CFLAGS) $(ENCLAVE_CFLAGS) $(RustEnclave_Include_Paths)
RustEnclave_Link_Flags = -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) -L $(SGX_SSL_LIBRARY_PATH) \
	-Wl,--whole-archive -l$(Trts_Library_Name) -lsgx_tsgxssl -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tstdc -l$(Service_Library_Name) -l$(Crypto_Library_Name) $(RustEnclave_Link_Libs) -lsgx_tsgxssl_crypto -lsgx_pthread -lsgx_tcxx -Wl,--end-group \
	-Wl,--version-script=enclave/Enclave.lds \
	$(ENCLAVE_LDFLAGS)

RustEnclave_Name = enclave/enclave.so
Signed_RustEnclave_Name = ${CUSTOM_BIN_PATH}/enclave.signed.so

.PHONY: all
all: $(App_Name) $(Signed_RustEnclave_Name) run run-ring

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) enclave/Enclave.edl
	$(SGX_EDGER8R) --trusted enclave/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --trusted-dir enclave
	$(SGX_EDGER8R) --untrusted enclave/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --untrusted-dir app
	@echo "GEN  =>  $(Enclave_EDL_Files)"

######## App Objects ########

app/Enclave_u.o: $(Enclave_EDL_Files)
	@$(CC) $(App_C_Flags) -c app/Enclave_u.c -o $@
	@echo "CC   <=  $<"

$(App_Enclave_u_Object): app/Enclave_u.o
	$(AR) rcsD $@ $^
	mkdir -p ${CUSTOM_LIBRARY_PATH}
	cp $(App_Enclave_u_Object) ${CUSTOM_LIBRARY_PATH}

$(App_Name): $(App_Enclave_u_Object) $(App_SRC_Files) $(CUSTOM_BIN_PATH)
	@cd app && SGX_SDK=$(SGX_SDK) cargo build $(App_Rust_Flags) 2>&1 | grep -v 'skipping duplicate package'
	@echo "Cargo  =>  $@"
	mkdir -p ${CUSTOM_BIN_PATH}
	cp $(App_Rust_Path)/app ${CUSTOM_BIN_PATH}

######## Enclave Objects ########

enclave/Enclave_t.o: $(Enclave_EDL_Files)
	@$(CC) $(RustEnclave_Compile_Flags) -c enclave/Enclave_t.c -o $@
	@echo "CC   <=  $<"

$(RustEnclave_Name): enclave enclave/Enclave_t.o
	@$(CXX) enclave/Enclave_t.o -o $@ $(RustEnclave_Link_Flags)
	@echo "LINK =>  $@"

$(Signed_RustEnclave_Name): $(RustEnclave_Name)
	mkdir -p ${CUSTOM_BIN_PATH}
	@$(SGX_ENCLAVE_SIGNER) sign -key enclave/Enclave_private.pem -enclave $(RustEnclave_Name) -out $@ -config enclave/Enclave.config.xml
	@echo "SIGN =>  $@"

$(CUSTOM_BIN_PATH):
	@echo "Folder $(CUSTOM_BIN_PATH) does not exist"
	mkdir -p $@

.PHONY: enclave
enclave:
	$(MAKE) -C ./enclave/ SGX_DEBUG=${SGX_DEBUG} CUSTOM_LIBRARY_PATH=${CUSTOM_LIBRARY_PATH}


.PHONY: clean
clean:
	@rm -f $(App_Name) $(RustEnclave_Name) $(Signed_RustEnclave_Name) enclave/*_t.* app/*_u.* lib/*.a
	@rm -rf ${CUSTOM_BIN_PATH} ${CUSTOM_LIBRARY_PATH}
	@cargo clean && rm -f Cargo.lock
	@cd app-ring; cargo clean; rm -f Cargo.lock

run:
	cd ${CUSTOM_BIN_PATH}; ./app
run-ring:
	cd app-ring; cargo run -q --release
bench:
	make bench-sim 2>&1 | grep --color='none' -E '(_enclave|iters)' | grep -Ev '(started|completed|_test)'
	make bench-hw 2>&1 | grep --color='none' -E '(_enclave|iters)' | grep -Ev '(started|completed|_test)'
bench-sim:
	make -B SGX_MODE=SIM
bench-hw:
	make -B SGX_MODE=HW
NAME=$(basename $(notdir ${CURDIR}))
BUILD_ARGS = --build-arg REPO_BASE=${REPO_BASE} --build-arg HTTPS_PROXY=${HTTPS_PROXY}
# only add sgx related params if sgx device exists
ifneq ($(wildcard /dev/sgx_enclave),)
    sgx_dev_params =-v /dev/sgx_enclave:/dev/sgx/enclave -v /dev/sgx_provision:/dev/sgx/provision -v /var/run/aesmd:/var/run/aesmd
endif
builder=docker
shell:
	${builder} images | grep -q ${NAME} || ${builder} build -t ${NAME} ${BUILD_ARGS} docker
	${builder} run --rm -it ${sgx_dev_params} -e https_proxy=${HTTPS_PROXY} -v ${CURDIR}:/${NAME} -w /${NAME} ${NAME}
