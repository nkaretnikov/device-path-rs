FROM ubuntu:18.04

# Update.
RUN apt-get update

# Install the dependencies.
RUN apt-get install -y build-essential git uuid-dev iasl nasm \
  python3 bc python3-distutils

# Build OVMF.
ARG EDK2_URL=https://github.com/tianocore/edk2.git
ARG EDK2_BRANCH=edk2-stable201905
ARG EDK2_DIR=/edk2
ARG BUILD_DIR=$EDK2_DIR/Build/OvmfX64/DEBUG_GCC5/FV
ARG OVMF_CODE=$BUILD_DIR/OVMF_CODE.fd
ARG OVMF_VARS=$BUILD_DIR/OVMF_VARS.fd
ARG NUM_THREADS=4
ARG OVMF_DIR=/ovmf

RUN git clone -b $EDK2_BRANCH --depth=1 $EDK2_URL $EDK2_DIR && \
  cd $EDK2_DIR && \
  git submodule update --init --recursive && \
  OvmfPkg/build.sh -n $NUM_THREADS -a X64 && \
  mkdir $OVMF_DIR && \
  cp $OVMF_CODE $OVMF_VARS $OVMF_DIR
