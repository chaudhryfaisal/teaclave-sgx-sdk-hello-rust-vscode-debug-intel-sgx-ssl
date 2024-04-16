ARG REPO_BASE
ARG IMG_BASE=oraclelinux:8.6
FROM ${REPO_BASE}docker.io/library/${IMG_BASE} AS base
ARG HTTPS_PROXY
ENV http_proxy=$HTTPS_PROXY https_proxy=$HTTPS_PROXY HTTP_PROXY=$HTTPS_PROXY HTTPS_PROXY=$HTTPS_PROXY
#ADD custom_ca /etc/pki/ca-trust/source/anchors/
#RUN update-ca-trust
RUN echo "http_proxy=$HTTPS_PROXY https_proxy=$HTTPS_PROXY HTTP_PROXY=$HTTPS_PROXY HTTPS_PROXY=$HTTPS_PROXY"
RUN curl https://google.com # test ssl

FROM base AS cargo
ARG RUST_TOOLCHAIN='nightly-2022-10-22'
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile complete --default-toolchain ${RUST_TOOLCHAIN}

FROM base AS sgx_sdk
RUN yum install -y make binutils
ARG SGX_SDK_SHORT=2.21
ARG SGX_SDK_DISTRO=rhel8.6-server
ARG SGX_SDK_FULL=2.21.100.1
ARG SGX_SDK_URL="https://download.01.org/intel-sgx/sgx-linux/${SGX_SDK_SHORT}/distro/${SGX_SDK_DISTRO}/sgx_linux_x64_sdk_${SGX_SDK_FULL}.bin"
RUN echo ${SGX_SDK_URL} && curl -L ${SGX_SDK_URL} -o /tmp/sdk.bin && rm -rf /opt/intel/sgxsdk && printf "no\n/opt/intel\n" | sh /tmp/sdk.bin && rm -f /tmp/sdk.bin

FROM base AS sgx_rpm_rpeo
ARG SGX_SDK_SHORT=2.21
ARG SGX_SDK_DISTRO=rhel8.6-server
ARG SGX_RPM_REPO="https://download.01.org/intel-sgx/sgx-linux/${SGX_SDK_SHORT}/distro/${SGX_SDK_DISTRO}/sgx_rpm_local_repo.tgz"
RUN mkdir -p /opt/intel && curl -sSL ${SGX_RPM_REPO} | tar -xzf - -C /opt/intel \
    && yum-config-manager --add-repo file:///opt/intel/sgx_rpm_local_repo

FROM base AS code_sgx_dcap
ARG DCAP_VER=1.18
ARG GIT_SGX_DCAP="https://github.com/intel/SGXDataCenterAttestationPrimitives/archive/refs/tags/dcap_${DCAP_VER}_reproducible.tar.gz"
RUN echo ${GIT_SGX_DCAP} && curl -sSL ${GIT_SGX_DCAP}  | tar -xzf - -C /tmp --strip-components 1 && \
    mkdir -p /code/sgx_dcap/${DCAP_VER} && mv /tmp/SampleCode /code/sgx_dcap/${DCAP_VER}

FROM base AS sgx_build_env
# Install epel-release, Development Tools, utils
RUN yum install -y epel-release && \
    yum groupinstall -y "Development Tools" --nobest &&  \
    yum install -y git wget which llvm clang make vim

## cargo ( takes long time to copy files over )
COPY --from=cargo /root/.cargo /root/.cargo
COPY --from=cargo /root/.rustup /root/.rustup

### sgx_rpm_rpeo
COPY --from=sgx_rpm_rpeo /opt/intel/sgx_rpm_local_repo /opt/intel/sgx_rpm_local_repo

RUN yum-config-manager --add-repo file:///opt/intel/sgx_rpm_local_repo && \
    echo 'gpgcheck=0' >> /etc/yum.repos.d/opt_intel_sgx_rpm_local_repo.repo
ARG SGX_RPM_PACKAGES="libsgx-enclave-common-devel libsgx-dcap-default-qpl-devel libsgx-dcap-ql-devel libsgx-dcap-quote-verify-devel libsgx-quote-ex-devel libsgx-launch-devel libsgx-urts-debuginfo"
RUN yum install --nogpgcheck -y ${SGX_RPM_PACKAGES}

## sgx_sdk
COPY --from=sgx_sdk /opt/intel/sgxsdk /opt/intel/sgxsdk

# Configure
ENV SGX_SDK=/opt/intel/sgxsdk GOROOT=/root/.go CARGO_HOME=/root/.cargo RUSTUP_HOME=/root/.rustup
ENV PATH=$PATH:$GOROOT/bin:$GOPATH/bin:$SGX_SDK/bin:$SGX_SDK/bin/x64:/root/.cargo/bin \
    LD_LIBRARY_PATH=$SGX_SDK/sdk_libs

# RESET ENV for runtime
ENV http_proxy='' https_proxy='' HTTP_PROXY='' HTTPS_PROXY=''