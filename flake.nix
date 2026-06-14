{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-26.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    mkflake.url = "github:jonascarpay/mkflake";
  };

  outputs = { nixpkgs, mkflake, rust-overlay, ... }: mkflake.lib.mkflake {
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
        rust-env = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-analyzer"
            "clippy"
            "rustfmt"
            "rust-src"
          ];
          targets = [
            "x86_64-unknown-linux-musl"
          ];
        };


        freqle-pkg = { rustPlatform }: rustPlatform.buildRustPackage {
          pname = "freqle";
          version = "0.1.0";
          src = ./.;
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
