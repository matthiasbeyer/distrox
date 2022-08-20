{
  description = "distrox";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.05";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustTarget = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
        unstableRustTarget = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "miri" ];
        });
        craneLib = (crane.mkLib pkgs).overrideToolchain rustTarget;

        tomlInfo = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };

        distrox-lib-deps = craneLib.buildDepsOnly {
          src = ./.;
          inherit (tomlInfo) pname version;
          cargoExtraArgs = "--package distrox-lib";
          nativeBuildInputs = nativeBuildPkgs;
          buildInputs = xorgPkgs;
        };

        distrox-cli-deps = craneLib.buildDepsOnly {
          src = ./.;
          inherit (tomlInfo) pname version;
          cargoExtraArgs = "--package distrox-cli";
          nativeBuildInputs = nativeBuildPkgs;
          buildInputs = xorgPkgs;
        };

        distrox-gui-deps = craneLib.buildDepsOnly {
          src = ./.;
          inherit (tomlInfo) pname version;
          cargoExtraArgs = "--package distrox-gui";
          nativeBuildInputs = nativeBuildPkgs;
          buildInputs = xorgPkgs;
        };


        nativeBuildPkgs = with pkgs; [
          curl
          gcc
          openssl
          pkgconfig
          which
          zlib

          freetype
          expat
          protobuf
        ];

        xorgPkgs = with pkgs.xorg; [
          libXcursor
          libXfont2
          # libXpm
          # libXtst
          # libxshmfence
          # libXft
          libXrandr
          libXext
          # libXinerama
          # libXrender
          # libXxf86misc
          # libxcb
          libX11
          # libXcomposite
          libXfont
          libXi
          # libXt
          # libxkbfile

          pkgs.libGL
        ];

      in
      rec {
        checks = {
          distrox-cli = packages.distrox-cli;
          distrox-gui = packages.distrox-gui;
          distrox-lib = packages.distrox-lib;

          # distrox-cli-clippy = craneLib.cargoClippy {
          #   inherit src;
          #   cargoArtifacts = packages.distrox-cli;
          #   cargoClippyExtraArgs = "-- --deny warnings";
          # };
          # distrox-gui-clippy = craneLib.cargoClippy {
          #   inherit src;
          #   cargoArtifacts = packages.distrox-gui;
          #   cargoclippyExtraArgs = "-- --deny warnings";
          # };
          distrox-lib-clippy = craneLib.cargoClippy {
            src = ./.;
            cargoExtraArgs = "--package distrox-lib";
            cargoArtifacts = distrox-lib-deps;
            cargoclippyExtraArgs = "-- --deny warnings";
          };

          distrox-cli-fmt = craneLib.cargoFmt {
            src = ./.;
            cargoExtraArgs = "--package distrox-cli";
          };
          distrox-lib-fmt = craneLib.cargoFmt {
            src = ./.;
            cargoExtraArgs = "--package distrox-lib";
          };
          distrox-gui-fmt = craneLib.cargoFmt {
            src = ./.;
            cargoExtraArgs = "--package distrox-gui";
          };
        };

        packages = {
          distrox-lib = craneLib.buildPackage {
            src = ./.;
            inherit (tomlInfo) version;
            pname = "distrox-lib";
            cargoExtraArgs = "--package distrox-lib";
            cargoArtifacts = distrox-lib-deps;
            nativeBuildInputs = nativeBuildPkgs;
            buildInputs = xorgPkgs;
            doCheck = false;
          };

          distrox-cli = craneLib.buildPackage {
            src = ./.;
            inherit (tomlInfo) version;
            pname = "distrox-cli";
            cargoExtraArgs = "--package distrox-cli";
            cargoArtifacts = distrox-cli-deps;
            nativeBuildInputs = nativeBuildPkgs;
            buildInputs = xorgPkgs ++ [ packages.distrox-lib ];
            doCheck = false;
          };

          distrox-gui = craneLib.buildPackage {
            src = ./.;
            inherit (tomlInfo) version;
            pname = "distrox-gui";
            cargoExtraArgs = "--package distrox-gui";
            cargoArtifacts = distrox-gui-deps;
            nativeBuildInputs = nativeBuildPkgs;
            buildInputs = xorgPkgs ++ [ packages.distrox-lib ];
            doCheck = false;
          };

          default = packages.distrox-gui;
        };

        apps = {
          distrox-gui = flake-utils.lib.mkApp {
            name = "distrox-gui";
            drv = packages.distrox-gui;
          };
          distrox-cli = flake-utils.lib.mkApp {
            name = "distrox-cli";
            drv = packages.distrox-cli;
          };
          apps.default = apps.distrox-gui;
        };

        devShells = {
          distrox = pkgs.mkShell {
            LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
            PROTOC          = "${pkgs.protobuf}/bin/protoc";
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath xorgPkgs;

            buildInputs = nativeBuildPkgs ++ xorgPkgs;

            nativeBuildInputs = nativeBuildPkgs ++ [
              rustTarget
              #unstableRustTarget

              pkgs.cargo-msrv
              pkgs.cargo-deny
              pkgs.cargo-expand
              pkgs.cargo-bloat
              pkgs.cargo-fuzz

              pkgs.gitlint
            ];
          };

          default = devShells.distrox;
        };
      }
    );
}

