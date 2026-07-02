{
  description = "A simple rust flake using rust-overlay and craneLib";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    nix-github-actions = {
      url = "github:nix-community/nix-github-actions";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    crates-io-index = {
      url = "git+https://github.com/rust-lang/crates.io-index?shallow=1";
      flake = false;
    };
    crates-nix = {
      url = "github:uttarayan21/crates.nix";
      inputs.crates-io-index.follows = "crates-io-index";
    };
  };

  outputs = {
    self,
    crane,
    flake-utils,
    nixpkgs,
    rust-overlay,
    advisory-db,
    nix-github-actions,
    crates-nix,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };
        inherit (pkgs) lib;
        # cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        # name = cargoToml.package.name;
        name = "llmproxy";

        stableToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = ["x86_64-unknown-linux-gnu" "wasm32-unknown-unknown"];
        };
        stableToolchainWithLLvmTools = stableToolchain.override {
          extensions = ["rust-src" "llvm-tools"];
        };
        stableToolchainWithRustAnalyzer = stableToolchain.override {
          extensions = ["rust-src" "rust-analyzer"];
        };
        # craneLib = (crane.mkLib pkgs).overrideToolchain stableToolchain;

        rustToolchainFor = p:
          p.rust-bin.stable.latest.default.override {
            # Set the build targets supported by the toolchain,
            # wasm32-unknown-unknown is required for trunk.
            targets = ["wasm32-unknown-unknown"];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        craneLibLLvmTools = (crane.mkLib pkgs).overrideToolchain stableToolchainWithLLvmTools;
        crates = crates-nix.mkLib {inherit pkgs;};
        lockedCrateVersion = crateName: lockFile: ((builtins.elemAt (builtins.filter (item: item.name == crateName) (builtins.fromTOML (builtins.readFile lockFile)).package)) 0).version;

        src = let
          filterBySuffix = path: exts: lib.any (ext: lib.hasSuffix ext path) exts;
          sourceFilters = path: type: (craneLib.filterCargoSources path type) || filterBySuffix path [".html" ".sql"];
        in
          lib.cleanSourceWith {
            filter = sourceFilters;
            src = ./.;
          };
        nativeArgs = {
          inherit src;
          pname = name;
          stdenv = p: p.clangStdenv;
          doCheck = false;
          buildInputs = with pkgs;
            []
            ++ (lib.optionals pkgs.stdenv.isDarwin [
              libiconv
              apple-sdk_26
            ]);
        };
        trunkArgs = {
          inherit src;
          pname = "frontend";
          cargoExtraArgs = "--package=frontend";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          cargoToml = ./frontend/Cargo.toml;
          # cargoLock = {
          #   lockFile = ./Cargo.lock;
          # };
          wasm-bindgen-cli = crates.buildCrate "wasm-bindgen-cli" {
            version = lockedCrateVersion "wasm-bindgen" ./Cargo.lock;
          };
        };
        cargoArtifacts = craneLib.buildPackage nativeArgs;
        cargoArtifactsWasm = craneLib.buildDepsOnly (
          trunkArgs
          // {
            doCheck = false;
          }
        );
      in {
        checks =
          {
            "${name}-clippy" = craneLib.cargoClippy (nativeArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });
            "${name}-docs" = craneLib.cargoDoc (nativeArgs // {inherit cargoArtifacts;});
            "${name}-fmt" = craneLib.cargoFmt {inherit src;};
            "${name}-toml-fmt" = craneLib.taploFmt {
              src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
            };
            # Audit dependencies
            "${name}-audit" = craneLib.cargoAudit {
              inherit src advisory-db;
            };

            # Audit licenses
            "${name}-deny" = craneLib.cargoDeny {
              inherit src;
            };
            "${name}-nextest" = craneLib.cargoNextest (nativeArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
              });
          }
          // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            "${name}-llvm-cov" = craneLibLLvmTools.cargoLlvmCov (nativeArgs // {inherit cargoArtifacts;});
          };

        packages = let
          frontend = craneLib.buildTrunkPackage (trunkArgs
            // {
              cargoArtifacts = cargoArtifactsWasm;
              preBuild = ''
                cd ./frontend
              '';
              postBuild = ''
                mv ./dist ..
                cd ..
              '';
            });
          pkg = craneLib.buildPackage (
            nativeArgs
            // {
              FRONTEND_ASSETS = frontend;
              inherit cargoArtifacts;
            }
          );
        in {
          "${name}" = pkg;
          "${name}-frontend" = frontend;
          default = pkg;
        };

        devShells = {
          default = pkgs.mkShell.override {stdenv = pkgs.clangStdenv;} (nativeArgs
            // {
              packages = with pkgs;
                [
                  stableToolchainWithRustAnalyzer
                  cargo-nextest
                  cargo-deny
                  trunk
                  sqlite
                  caddy
                  lsof
                  cargo-watch
                ]
                ++ (lib.optionals pkgs.stdenv.isDarwin [
                  apple-sdk_26
                ]);
              shellHook = ''
                echo "Welcome to the development shell for ${name}!"
              '';
            });
        };
      }
    )
    // {
      githubActions = nix-github-actions.lib.mkGithubMatrix {
        checks = nixpkgs.lib.getAttrs ["x86_64-linux"] self.checks;
      };
      nixosModules = rec {
        llmproxy = ./nix/nixosModules/llmproxy.nix;
        default = llmproxy;
      };
    };
}
