#!/bin/bash


# It is assumed you have Rust and DFX installed...

set -e

##################################
############# prepare VARs

export SDK_DIR=$HOME/.cache/wasi-sdk
export SDK_VERSION=27
export OS=`uname -s`
export ARCH=`uname -m`


# Normalize OS names
if [[ "$OS" == "Darwin" ]]; then
    OS="macos"
elif [[ "$OS" == "Linux" ]]; then
    OS="linux"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

# Normalize architecture names
if [[ "$ARCH" == "x86_64" ]]; then
    ARCH="x86_64"
elif [[ "$ARCH" == "arm64" || "$ARCH" == "aarch64" ]]; then
    ARCH="arm64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi


export WASI_DIR=wasi-sdk-$SDK_VERSION.0-$ARCH-$OS
export WASI_SDK=$SDK_DIR/$WASI_DIR

if [[ "$1" == "--sdk" ]]; then
  echo $WASI_SDK
  exit 0
fi

export SRC=https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$SDK_VERSION/$WASI_DIR.tar.gz

if [[ "$OS" == "linux" && "$ARCH" == "x86_64" ]] ||
   [[ "$OS" == "linux" && "$ARCH" == "arm64" ]] ||
   [[ "$OS" == "macos" && "$ARCH" == "x86_64" ]] ||
   [[ "$OS" == "macos" && "$ARCH" == "arm64" ]]; then
    echo "✅ Detected supported platform: $OS-$ARCH"
else
    echo "❌ Unsupported OS/Architecture combination: $OS-$ARCH"
    exit 1
fi

##################################
############# add wasm32-wasi target

echo "Installing wasm32-wasip1 target..."
rustup target add wasm32-wasip1

##################################
############# install wasi2ic

echo "Installing wasi2ic..."
cargo install wasi2ic

##################################
############# download WASI-SDK

if [ ! -d "$WASI_SDK" ]; then

    echo "Downloading WASI-SDK..."
    
    mkdir -p $SDK_DIR

    curl -L -o $SDK_DIR/wasi-sdk.tar.gz $SRC

    tar -xzf $SDK_DIR/wasi-sdk.tar.gz -C $SDK_DIR

    [ -f "$SDK_DIR/wasi-sdk.tar.gz" ] && rm "$SDK_DIR/wasi-sdk.tar.gz"

else
    echo "WASI-SDK found in: $WASI_SDK ..."
fi

##################################
############# Update .bashrc

CONFIG_LINES=(
  "export WASI_SDK=$WASI_SDK"
  'export PATH="$WASI_SDK/bin:$PATH"'
)

BASHRC="$HOME/.bashrc"

# Check which lines are missing
missing_lines=()
for line in "${CONFIG_LINES[@]}"; do
  if ! grep -Fxq "$line" "$BASHRC"; then
    missing_lines+=("$line")
  fi
done

if [ ${#missing_lines[@]} -eq 0 ]; then
  echo "✅ .bashrc is ready"
  exit 0
fi

AUTO_CONFIRM=false
if [[ "$1" == "-y" || "$1" == "--yes" ]]; then
  AUTO_CONFIRM=true
fi

if $AUTO_CONFIRM; then
  RESPONSE="Y"
else
  read -p "Do you want to update yor .bashrc? [y/N] " RESPONSE
fi

if [[ "$RESPONSE" =~ ^[Yy]$ ]]; then
  for line in "${CONFIG_LINES[@]}"; do
    echo "$line" >> "$BASHRC"
  done
  echo "" >> "$BASHRC"

  echo "✅ .bashrc updated"

  source ~/.bashrc

else
  echo "ℹ️ Skipped modifying .bashrc,"
  echo 'To enable compilation, make sure you point $WASI_SDK to wasi-sdk installation and the wasi-oriented clang compiler is available on the path:'
  echo 'export WASI_SDK=/opt/wasi-sdk'
  echo 'export PATH=$WASI_SDK/bin'

fi
