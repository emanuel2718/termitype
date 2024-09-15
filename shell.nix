{
  pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz") { },
}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustc
    cargo
    gcc
    rustfmt
    clippy
    rust-analyzer
  ];

  # Some crates require .cargo/bin to be on the path
  shellHook = ''
    export PATH=$PATH:$HOME/.cargo/bin
  '';

  # Certain Rust tools won't work without this
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}

