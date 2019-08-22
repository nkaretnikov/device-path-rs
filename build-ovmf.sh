#!/usr/bin/env bash

set -euxo pipefail

CONTAINER_NAME=ovmf
OVMF_DIR=/ovmf

docker build -t $CONTAINER_NAME .
CONTAINER_ID=$(docker run -d -it $CONTAINER_NAME)

docker cp $CONTAINER_ID:$OVMF_DIR/OVMF_CODE.fd .
docker cp $CONTAINER_ID:$OVMF_DIR/OVMF_VARS.fd .

docker stop $CONTAINER_ID
