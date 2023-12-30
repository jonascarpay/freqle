{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.05";
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
        pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlay ]; };
        rust = pkgs.rust-bin.selectLatestNightlyWith
          (toolchain: toolchain.default.override {
            extensions = [ "rust-analyzer" ];
          });
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rust
          ];
        };
      };
  };
}
