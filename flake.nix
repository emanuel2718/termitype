{
  description = "Another TUI typing game, inspired by a certain typing test you might know.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.stable.latest.default.override {
              extensions = [ "clippy" "rustfmt" "rust-src" ];
            })
            rust-analyzer
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "termitype";
          version = "0.0.6";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      });
} 
