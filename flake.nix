{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    mkflake.url = "github:jonascarpay/mkflake";
  };

  outputs = { nixpkgs, mkflake, rust-overlay, ... }: mkflake.lib.mkflake {
    toplevel = {
      # top-level outputs, such as library functions and
      # overlays go here
    };
    perSystem = system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            (final: prev: {
              freqle = final.callPackage freqle-pkg { };
            })
          ];
        };
        rust-env = pkgs.rust-bin.selectLatestNightlyWith
          (toolchain: toolchain.default.override {
            extensions = [
              "rust-analyzer"
              "clippy"
              "rustfmt"
              "rust-src"
            ];
            targets = [
              "x86_64-unknown-linux-musl"
            ];
          });


        freqle-pkg = { rustPlatform }: rustPlatform.buildRustPackage {
          pname = "freqle";
          version = "0.1";
          src = ./.;
          # cargoHash = "sha256-qqmTfmsLDTpU2Dsz+wUD4mFjWE0vm4F/wW2J21HYuWs=";
          cargoLock.lockFile = ./Cargo.lock;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rust-env
          ];
        };
        packages = rec {
          default = freqle;
          freqle = pkgs.freqle;
          freqle-static = pkgs.pkgsStatic.freqle;

        };
      };
  };
}
