{
  description = "Rust + htmx + tailwind + nix + redb + twind demo web app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flakebox = {
      url = "github:rustshop/flakebox?rev=b07a9f3d17d400464210464e586f76223306f62d";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flakebox }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;

          overlays = [
            (final: prev: {
              # mold wrapper from https://discourse.nixos.org/t/using-mold-as-linker-prevents-libraries-from-being-found/18530/5
              mold-wrapped =
                let
                  bintools-wrapper = "${nixpkgs}/pkgs/build-support/bintools-wrapper";
                in
                prev.symlinkJoin {
                  name = "mold";
                  paths = [ prev.mold ];
                  nativeBuildInputs = [ prev.makeWrapper ];
                  suffixSalt = prev.lib.replaceStrings [ "-" "." ] [ "_" "_" ] prev.targetPlatform.config;
                  postBuild = ''
                    for bin in ${prev.mold}/bin/*; do
                      rm $out/bin/"$(basename "$bin")"

                      export prog="$bin"
                      substituteAll "${bintools-wrapper}/ld-wrapper.sh" $out/bin/"$(basename "$bin")"
                      chmod +x $out/bin/"$(basename "$bin")"

                      mkdir -p $out/nix-support
                      substituteAll "${bintools-wrapper}/add-flags.sh" $out/nix-support/add-flags.sh
                      substituteAll "${bintools-wrapper}/add-hardening.sh" $out/nix-support/add-hardening.sh
                      substituteAll "${bintools-wrapper}/../wrapper-common/utils.bash" $out/nix-support/utils.bash
                    done
                  '';
                };
            })
          ];
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
                nativeBuildInputs = [ pkgs.mold-wrapped ];
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
            nativeBuildInputs = [ pkgs.mold-wrapped ];
          };
        };
      }
    );
}
