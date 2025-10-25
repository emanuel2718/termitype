#!/bin/bash

set -e

VERSION=$(curl -s https://api.github.com/repos/emanuel2718/termitype/releases/latest | grep '"tag_name":' | head -1 | cut -d'"' -f4)

case $(uname -s) in
    Darwin) os="apple-darwin" ;;
    Linux) os="unknown-linux-gnu" ;;
    MINGW*|MSYS*|CYGWIN*) os="pc-windows-msvc" ;;
    *) echo "Unsupported OS: $(uname -s)"; exit 1 ;;
esac

case $(uname -m) in
    x86_64) arch="x86_64" ;;
    aarch64) arch="aarch64" ;;
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

if [[ "$os" == "pc-windows-msvc" ]]; then
    echo "Installing to ~/bin..."
    mkdir -p ~/bin
    cp "$temp/$binary" ~/bin/
    echo "Termitype installed successfully!"
    echo "Add ~/bin to your PATH if not already."
else
    echo "Installing to /usr/local/bin..."
    sudo cp "$temp/$binary" /usr/local/bin/
    sudo chmod +x /usr/local/bin/$binary
    echo "Termitype installed successfully!"
fi
