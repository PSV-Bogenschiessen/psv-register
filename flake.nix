{
  description = "A basic flake with a shell";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
          (import nix/overlay.nix)
        ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      rec {
        devShells.default = pkgs.mkShell {
          DATABASE_URL = "database.sqlite";
          nativeBuildInputs = [
            #shared
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [
                "wasm32-unknown-unknown"
                (pkgs.rust.toRustTargetSpec pkgs.stdenv.hostPlatform)
              ];
            })
            pkgs.rust-analyzer

            #backend
            pkgs.diesel-cli
            pkgs.openssl.dev
            pkgs.cargo-watch

            #frontend
            # pkgs.cargo-generate
            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.pkg-config
          ] ++ pkgs.lib.lists.optional pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ];
          buildInputs = [
            pkgs.sqlite.dev
          ];
        };

        packages = rec {
          frontend = import nix/frontend.nix { inherit pkgs; };
          backend = import nix/backend.nix { inherit pkgs frontend; };
          default = backend;
        };

        formatter = pkgs.nixpkgs-fmt;

        nixosModules.psv-registration = import nix/module.nix "psv-registration-cup" packages.default;
      });
}
