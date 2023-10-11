{
  description = "Rust + htmx + tailwind + nix + redb + twind demo web app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flakebox = {
      url = "github:rustshop/flakebox?rev=1e4cce8057d7d68798147ab18cf7ad2ab16506b8";
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
                nativeBuildInputs = [ ];
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
            packages = [ ];
            nativeBuildInputs = [ ];
          };
        };
      }
    );
}
