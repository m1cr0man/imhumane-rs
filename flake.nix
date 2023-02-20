{
  description = "I'm Humane human validator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        devDependencies = with pkgs; [
          openssl
          pkgconfig
          (rust-bin.nightly."2023-02-17".default.override {
            extensions = [ "rust-src" "rustfmt" ];
          })
          lldb
          rust-analyzer
          cmake
          libev
          uthash
        ];
      in
      rec {
        devShells.default = pkgs.mkShell {
          buildInputs = devDependencies;
        };

        packages.venv = pkgs.symlinkJoin {
          name = "imhumane-rs-venv";
          paths = devDependencies;
        };

        devShell = devShells.default;
      }
    );
}
