#!/bin/bash

# assign the path to your WASI_SDK installation, 
# change this line if your installation is different
export WASI_SDK=/opt/wasi-sdk

# provide path to the wasi-oriented clang installation
export PATH=$WASI_SDK/bin:$PATH

# run DFX
dfx start --clean --background

# build and deploy the canister
dfx deploy
