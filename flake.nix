{
  description = "virtual environments";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, flake-utils, devshell, fenix, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system: {
      defaultPackage = let pkgs = import nixpkgs { inherit system; };
      in pkgs.rustPlatform.buildRustPackage {
        pname = "watch-changes";
        version = "0.3.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
      devShell = let
        pkgs = import nixpkgs {
          inherit system;

          overlays = [ devshell.overlay ];
        };
      in pkgs.devshell.mkShell {
        imports = [ (pkgs.devshell.importTOML ./devshell.toml) ];
      };
    });
}
