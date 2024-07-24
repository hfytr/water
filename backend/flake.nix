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
      in {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            openssl
            postgresql
            pkg-config
            rust-bin.stable.latest.default
            fish
            sqlx-cli
          ];

          PGDATA = ".pgdata";
          shellHook = ''
            fish scripts/init_db.fish
            function on_exit {
                pg_ctl --pgdata=$PGDATA stop
            }
            trap on_exit EXIT
            fish
          '';
        };
      }
    );
}
