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
        inherit (tomlInfo) pname version;
        src = ./.;

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;
        };

        distrox = craneLib.buildPackage {
          inherit cargoArtifacts src version;
        };

      in
      rec {
        checks = {
          inherit distrox;

          distrox-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src;
            cargoClippyExtraArgs = "-- --deny warnings";
          };

          distrox-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        packages.distrox = distrox;
        packages.default = packages.distrox;

        apps.distrox = flake-utils.lib.mkApp {
          name = "distrox";
          drv = distrox;
        };
        apps.default = apps.distrox;

        devShells.default = devShells.distrox;
        devShells.distrox = pkgs.mkShell {
          LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
          PROTOC          = "${pkgs.protobuf}/bin/protoc";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath xorgPackages;

          buildInputs = let
            buildPkgs = with pkgs; [
              cmake
              curl
              gcc
              openssl
              pkgconfig
              which
              zlib

              freetype
              expat
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
          in buildPkgs ++ xorgPkgs;

          nativeBuildInputs = [
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
      }
    );
}

