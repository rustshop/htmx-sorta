{
  description = "dpc's basic flake template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flakebox = {
      url = "github:rustshop/flakebox?rev=36b349dc4e6802a0a26bafa4baef1f39fbf4e870";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flakebox }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        lib = pkgs.lib;
        extLib = import ./nix/lib.nix { inherit lib; };

        flakeboxLib = flakebox.lib.${system} { };
        craneLib = flakeboxLib.craneLib;

        commonArgs =
          let
            staticFilesFilter = path: type: if type == "directory" then lib.hasPrefix "/static/" path else lib.hasPrefix "/static/" path;
          in
          {
            src = extLib.cleanSourceWithRel {
              src = builtins.path {
                name = "htmx-demo";
                path = ./.;
              };
              filter = path: type:
                (staticFilesFilter path type)
                ||
                (craneLib.filterCargoSources path type)
              ;
            };

            installCargoArtifactsMode = "use-zstd";

            buildInputs = [ ];

            nativeBuildInputs = builtins.attrValues
              {
                inherit (pkgs) lld mold;
              } ++ [ ];
          };
      in
      {
        packages.default = craneLib.buildPackage ({ } // commonArgs);

        devShells = {
          default = flakeboxLib.mkDevShell {
            packages = [ pkgs.mold ];
          };
        };
      }
    );
}
