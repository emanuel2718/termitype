{
  pkgs ? let
    rust-overlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
    nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz";
  in import nixpkgs {
    overlays = [ (import rust-overlay) ];
  },
}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    (rust-bin.stable."1.84.0".default.override {
      extensions = [ "clippy" "rustfmt" "rust-src" ];
    })
    gcc
    rust-analyzer
  ];

  # Some crates require .cargo/bin to be on the path
  shellHook = ''
    export PATH=$PATH:$HOME/.cargo/bin
  '';
}

