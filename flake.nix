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

        guiBuildInputs = (with pkgs; [
          alejandra
          appimagekit
          atk
          cairo
          dbus.lib
          dprint
          gdk-pixbuf
          glib.out
          gtk3
          harfbuzz
          libsoup
          nodejs-16_x
          openssl.out
          pango
          pkg-config
          treefmt
          webkitgtk
          zlib
        pkg-config
        ]) ++ (with pkgs.xorg; [
          libX11
          libXcomposite
          libXcursor
          libXext
          libXfont
          libXfont2
          libXft
          libXi
          libXinerama
          libXmu
          libXpm
          libXpresent
          libXrandr
          libXrender
          libXt
          libXtst
          libXxf86misc
          libXxf86vm
          libxcb
          libxkbfile
          libxshmfence

          pkgs.libGL
          pkgs.pkgconfig
        ]);
      in
      rec {
        devShells = {
          distrox = pkgs.mkShell {
            LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
            PROTOC          = "${pkgs.protobuf}/bin/protoc";
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath guiBuildInputs;

            buildInputs = nativeBuildPkgs ++ guiBuildInputs;

            nativeBuildInputs = nativeBuildPkgs ++ [
              rustTarget

              pkgs.cargo-msrv
              pkgs.cargo-deny
              pkgs.cargo-expand
              pkgs.cargo-bloat
              pkgs.cargo-fuzz
              pkgs.cargo-outdated
              pkgs.trunk

              pkgs.gitlint
            ];

            # Ugly as fuck, but I have debugged the issue of the devtools not
            # working for almost 10 hours and I did not find what needs to be in
            # XDG_DATA_DIRS for it to work. /run/current-system/sw/share makes
            # it work (on nixos only of course).
            # I know this makes the shell impure, but that's all I've got.
            # If someone knows how to do better, please submit a fix for this.
            shellHook = ''
              export XDG_DATA_DIRS="$XDG_DATA_DIRS:/run/current-system/sw/share"
            '';
          };

          default = devShells.distrox;
        };
      }
    );
}

