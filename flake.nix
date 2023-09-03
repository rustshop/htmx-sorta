{
  description = "dpc's basic flake template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        lib = pkgs.lib;
        extLib = import ./nix/lib.nix { inherit lib; };

        fenixChannel = fenix.packages.${system}.stable;
        fenixChannelNightly = fenix.packages.${system}.latest;

        fenixToolchain = (fenixChannel.withComponents [
          "rustc"
          "cargo"
          "clippy"
          "rust-analysis"
          "rust-src"
        ]);

        fenixToolchainRustfmt = (fenixChannelNightly.withComponents [
          "rustfmt"
        ]);

        craneLib = crane.lib.${system}.overrideToolchain fenixToolchain;

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

            buildInputs = [ ];

            nativeBuildInputs = builtins.attrValues
              {
                inherit (pkgs) lld;
              } ++ [ ];
          };
      in
      {
        packages. default = craneLib.buildPackage ({ } // commonArgs);

        devShells = {
          default = pkgs.mkShell {

            buildInputs = [ ] ++ commonArgs.buildInputs;

            nativeBuildInputs = builtins.attrValues
              {
                inherit (pkgs) cargo-watch;
                inherit fenixToolchain fenixToolchainRustfmt;
                inherit (pkgs) nixpkgs-fmt shellcheck rnix-lsp just;
                inherit (pkgs) lld parallel typos convco;
              } ++ [
              # This is required to prevent a mangled bash shell in nix develop
              # see: https://discourse.nixos.org/t/interactive-bash-with-nix-develop-flake/15486
              (pkgs.hiPrio pkgs.bashInteractive)
              pkgs.nodePackages.bash-language-server

            ] ++ commonArgs.nativeBuildInputs;

            shellHook = ''
              dot_git="$(git rev-parse --git-common-dir)"
              if [[ ! -d "$dot_git/hooks" ]]; then mkdir "$dot_git/hooks"; fi
              chmod +x .git/hooks/{pre-commit,commit-msg}
              for hook in misc/git-hooks/* ; do ln -sf "$(pwd)/$hook" "$dot_git/hooks/" ; done
              ${pkgs.git}/bin/git config commit.template $(pwd)/misc/git-hooks/commit-template.txt

              # if running in direnv
              if [ -n "''${DIRENV_IN_ENVRC:-}" ]; then
                # and not set DIRENV_LOG_FORMAT
                if [ -n "''${DIRENV_LOG_FORMAT:-}" ]; then
                  >&2 echo "💡 Set 'DIRENV_LOG_FORMAT=\"\"' in your shell environment variables for a cleaner output of direnv"
                fi
              fi

              >&2 echo "💡 Run 'just' for a list of available 'just ...' helper recipes"
            '';
          };
        };
      }
    );
}
