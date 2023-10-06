{
  description = "dpc's basic flake template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flakebox = {
      url = "github:rustshop/flakebox?rev=b07a9f3d17d400464210464e586f76223306f62d";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flakebox }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        flakeboxLib = flakebox.lib.${system} { };
        craneLib = flakeboxLib.craneLib;

        src = flakeboxLib.filter.filterSubdirs {
          root = builtins.path {
            name = "htmx-demo";
            path = ./.;
          };
          dirs = [
            "Cargo.toml"
            "Cargo.lock"
            ".cargo"
            "src"
            "static"
          ];
        };
      in
      {
        packages.default = craneLib.buildPackage { inherit src; };

        devShells = {
          default = flakeboxLib.mkDevShell {
            packages = [ pkgs.mold ];
          };
        };
      }
    );
}
