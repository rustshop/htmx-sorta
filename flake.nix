{
  description = "Rust + htmx + tailwind + nix + redb + twind demo web app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    flakebox = {
      url = "github:rustshop/flakebox?rev=d60061baec213962a897fe117e19ce1996c7f0a2";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flakebox }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        projectName = "htmx-sorta";

        pkgs = nixpkgs.legacyPackages.${system};

        flakeboxLib = flakebox.lib.${system} {
          config = {
            github.ci.buildOutputs = [ ".#ci.${projectName}" ];
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
                pname = projectName;
                src = buildSrc;
                nativeBuildInputs = [ ];
              });
            in
            {
              ${projectName} = craneLib.buildPackage { };
            });
      in
      {
        packages.default = multiBuild.${projectName};

        legacyPackages = multiBuild;

        devShells = {
          default = flakeboxLib.mkDevShell {
            packages = [ pkgs.tailwindcss ];
            nativeBuildInputs = [ ];
          };
        };
      }
    );
}
