{
  description = "Dev environment for rust";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/22.05";
  inputs.devshell = {
    url = "github:numtide/devshell";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.flake-utils = {
    url = "github:numtide/flake-utils";
    inputs.nixpkgs.follows = "nixpkgs";
  };
  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, flake-utils, devshell, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system: {
      defaultPackage =
        let pkgs = import nixpkgs { inherit system; };
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "watch-changes";
          version = "0.3.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      devShell =
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [ devshell.overlay rust-overlay.overlays.default ];
          };
        in
        pkgs.devshell.mkShell {
          imports = [ (pkgs.devshell.importTOML ./devshell.toml) ];
        };
    });
}
