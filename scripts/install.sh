#!/bin/bash

set -euo pipefail

PREFIX="/usr/local"

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Install Termitype from the latest GitHub release.

OPTIONS:
    -p, --prefix DIR    Install to DIR/bin instead of /usr/local/bin
    -u, --uninstall     Uninstall Termitype
    -h, --help          Show this help message
    -v, --version       Show version and exit

EXAMPLES:
    $0
    $0 --prefix ~/bin
    $0 --uninstall

EOF
}

check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is required but not installed."
        exit 1
    fi
}

check_command curl
check_command tar
check_command unzip

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--prefix)
            PREFIX="$2"
            shift 2
            ;;
        -u|--uninstall)
            UNINSTALL=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -v|--version)
            echo "Termitype installer version 1.0"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

if [[ "${UNINSTALL:-false}" == true ]]; then
    echo "Uninstalling Termitype from $PREFIX/bin..."
    if [[ -f "$PREFIX/bin/termitype" ]]; then
        sudo rm -f "$PREFIX/bin/termitype"
        echo "Termitype uninstalled successfully."
    else
        echo "Termitype not found in $PREFIX/bin."
    fi
    exit 0
fi

VERSION=$(curl -s https://api.github.com/repos/emanuel2718/termitype/releases/latest | grep '"tag_name":' | head -1 | cut -d'"' -f4)

case $(uname -s) in
    Darwin) os="apple-darwin" ;;
    Linux) os="unknown-linux-gnu" ;;
    MINGW*|MSYS*|CYGWIN*) os="pc-windows-msvc" ;;
    *) echo "Unsupported OS: $(uname -s)"; exit 1 ;;
esac

case $(uname -m) in
    x86_64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) echo "Unsupported architecture: $(uname -m)"; exit 1 ;;
esac

platform="${arch}-${os}"

temp=$(mktemp -d)
trap "rm -rf $temp" EXIT

if [[ "$os" == "pc-windows-msvc" ]]; then
    url="https://github.com/emanuel2718/termitype/releases/download/${VERSION}/termitype-${VERSION}-${platform}.zip"
    binary="termitype.exe"
    extract_cmd="unzip -q"
    archive_file="$temp/termitype.zip"
else
    url="https://github.com/emanuel2718/termitype/releases/download/${VERSION}/termitype-${VERSION}-${platform}.tar.gz"
    binary="termitype"
    extract_cmd="tar -xzf"
    archive_file="$temp/termitype.tar.gz"
fi

echo "Downloading Termitype ${VERSION}..."
curl -L "$url" -o "$archive_file"

echo "Extracting..."
$extract_cmd "$archive_file" -C "$temp"

binary_path=$(find "$temp" -name "$binary" -type f | head -1)
if [[ -z "$binary_path" ]]; then
    echo "Error: Binary '$binary' not found after extraction."
    exit 1
fi

if [[ "$os" == "pc-windows-msvc" ]]; then
    INSTALL_DIR="$HOME/bin"
else
    INSTALL_DIR="$PREFIX/bin"
fi

echo "Installing to $INSTALL_DIR..."

if [[ ! -d "$INSTALL_DIR" ]]; then
    if [[ "$INSTALL_DIR" == "$HOME/bin" ]]; then
        mkdir -p "$INSTALL_DIR"
    else
        sudo mkdir -p "$INSTALL_DIR"
    fi
fi

if [[ "$INSTALL_DIR" == "$HOME/bin" ]]; then
    cp "$binary_path" "$INSTALL_DIR/"
else
    sudo cp "$binary_path" "$INSTALL_DIR/"
fi

if [[ "$os" != "pc-windows-msvc" ]]; then
    sudo chmod +x "$INSTALL_DIR/$binary"
fi

echo "Termitype installed successfully!"

if ! echo "$PATH" | tr ':' '\n' | grep -q "^$INSTALL_DIR$"; then
    echo "Note: Add $INSTALL_DIR to your PATH if not already present."
fi
