{
  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustpkg = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" "rustfmt" ];
          targets = [ "arm-unknown-linux-gnueabihf" ];
        });
      in {
        RUST_BACKTRACE = 1;
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            openssl
            postgresql
            pkg-config
            rustpkg
            fish
            sqlx-cli
          ];

          shellHook = ''
            source scripts/init_db.sh
          '';
        };
      }
    );
}
