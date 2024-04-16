#!/bin/bash
#branch=3.0_Rev1 openssl=3.0.10 ./setup-intel-sgx-ssl.sh
#branch=3.0_Rev2 openssl=3.0.12 ./setup-intel-sgx-ssl.sh
branch=${branch:-3.0_Rev2}
openssl=${openssl:-3.0.12}
sdk=${sdk:-2.21}
work_dir=/tmp/intel-sgx-ssl
clone_dir=${work_dir}/intel-sgx-ssl-${branch}
Enclave_private=`realpath enclave/Enclave_private.pem`
echo "branch=$branch openssl=$openssl sdk=$sdk Enclave_private=$Enclave_private"

# Pre
mkdir -p ${work_dir}
test -f ${work_dir}/as.ld.objdump.r4.tar.gz || wget https://download.01.org/intel-sgx/sgx-linux/${sdk}/as.ld.objdump.r4.tar.gz -O ${work_dir}/as.ld.objdump.r4.tar.gz
test -d ${work_dir}/external/toolset || tar -xf ${work_dir}/as.ld.objdump.r4.tar.gz -C ${work_dir}
cp ${work_dir}/external/toolset/centos8/* /usr/local/bin
yum install -y perl-IPC-Cmd

source ${SGX_SDK}/environment
test -d ${clone_dir} || git clone --branch ${branch} https://github.com/intel/intel-sgx-ssl.git ${clone_dir}
test -f ${clone_dir}/openssl_source/openssl-${openssl}.tar.gz || wget https://www.openssl.org/source/openssl-${openssl}.tar.gz -P ${clone_dir}/openssl_source
cd ${clone_dir}/Linux
sed -i "s|openssl genrsa.*|cp ${Enclave_private} \$(Enclave_Test_Key)|" sgx/test_app/sgx_t.mk
make all test SGX_MODE=SIM
rm -rf /opt/intel/sgxssl # remove dir if it exists
make install 
find /opt/intel/sgxssl/lib64