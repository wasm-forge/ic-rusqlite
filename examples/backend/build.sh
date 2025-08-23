#!/bin/bash

# assign the path to your WASI_SDK_PATH installation, 
# change this line if your installation is different
#export WASI_SDK_PATH=/opt/wasi-sdk

# provide path to the wasi-oriented clang installation
#export PATH=$WASI_SDK_PATH/bin:$PATH

# run DFX
dfx start --clean --background

# build and deploy the canister
dfx deploy
