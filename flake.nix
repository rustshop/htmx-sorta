{
  description = "Rust + htmx + tailwind + nix + redb + twind demo web app";

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
        projectName = "htmx-sorta";

        flakeboxLib = flakebox.lib.${system} {
          config = {
            github.ci.buildOutputs = [ ".#ci.htmx-sorta" ];
          };
        };

        buildPaths = [
          "Cargo.toml"
          "Cargo.lock"
          ".cargo"
          "src"
          "static"
        ];

        buildSrc = flakeboxLib.filterSubPaths {
          root = builtins.path {
            name = projectName;
            path = ./.;
          };
          paths = buildPaths;
        };

        multiBuild =
          (flakeboxLib.craneMultiBuild { }) (craneLib':
            let
              craneLib = (craneLib'.overrideArgs {
                pname = "flexbox-multibuild";
                src = buildSrc;
              });
            in
            {
              htmx-sorta = craneLib.buildPackage { };
            });
      in
      {
        packages.default = multiBuild.htmx-sorta;

        legacyPackages = multiBuild;

        devShells = {
          default = flakeboxLib.mkDevShell {
            packages = [ pkgs.mold ];
          };
        };
      }
    );
}
