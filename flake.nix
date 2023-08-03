{
  description = "distrox";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.05";
    unstable-nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
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
    cargo-changelog = {
      url = "github:matthiasbeyer/cargo-changelog";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
        crane.follows = "crane";
        rust-overlay.follows = "rust-overlay";
      };
    };
  };

  outputs =
    { self
    , nixpkgs
    , crane
    , flake-utils
    , rust-overlay
    , unstable-nixpkgs
    , cargo-changelog
    , ...
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      unstable = import unstable-nixpkgs {
        inherit system;
      };

      rustTarget = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      unstableRustTarget = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
        extensions = [ "rust-src" "miri" ];
      });
      craneLib = (crane.mkLib pkgs).overrideToolchain rustTarget;

      tomlInfo = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };

      nativeBuildInputs = with pkgs; [
        curl
        gcc
        openssl
        zlib
        pkg-config
      ];

      buildInputs = with pkgs; [
      ];

      guiBuildInputs = with pkgs; [
      ];

      src =
        let
          markdownFilter = path: _type: pkgs.lib.hasSuffix ".md" path;
          filterPath = path: type: builtins.any (f: f path type) [
            markdownFilter
            craneLib.filterCargoSources
            pkgs.lib.cleanSourceFilter
          ];
        in
        pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = filterPath;
        };

      distroxLibArtifacts = craneLib.buildDepsOnly {
        pname = "distrox-lib";
        inherit src;
        inherit nativeBuildInputs;
        inherit buildInputs;
        cargoExtraArgs = "-p distrox-lib --all-features";
      };

      distroxGuiArtifacts = craneLib.buildDepsOnly {
        pname = "distrox-gui";
        inherit src;
        inherit nativeBuildInputs;
        buildInputs = guiBuildInputs;
        cargoExtraArgs = "-p distrox-gui --all-features";
      };

      distroxCliArtifacts = craneLib.buildDepsOnly {
        pname = "distrox-cli";
        inherit src;
        inherit nativeBuildInputs;
        inherit buildInputs;
        cargoExtraArgs = "-p distrox-cli --all-features";
      };

      distrox-lib = craneLib.buildPackage {
        inherit (tomlInfo) version;
        pname = "distrox-lib";
        inherit src;
        inherit buildInputs;
        inherit nativeBuildInputs;
        cargoExtraArgs = "-p distrox-lib --all-features";
        cargoArtifacts = distroxLibArtifacts;

        # we use distrox-lib-tests to run tests with testcontainers
        doCheck = false;
      };

      distrox-lib-tests = craneLib.cargoNextest {
        inherit src;
        cargoArtifacts = distroxLibArtifacts;
        inherit nativeBuildInputs;
        inherit buildInputs;

        doNotLinkInheritedArtifacts = true;
      };

      distrox-gui = craneLib.buildPackage {
        pname = "distrox-gui";
        inherit (tomlInfo) version;
        inherit src;
        inherit nativeBuildInputs;
        cargoExtraArgs = "-p distrox-gui --all-features";
        cargoArtifacts = distroxGuiArtifacts;
      };

      distrox-cli = craneLib.buildPackage {
        pname = "distrox-cli";
        inherit (tomlInfo) version;
        inherit src;
        inherit nativeBuildInputs;
        cargoExtraArgs = "-p distrox-cli --all-features";
        cargoArtifacts = distroxCliArtifacts;
      };
    in
    rec {
      checks = {
        default = distrox-gui;

        inherit distrox-lib;
        inherit distrox-lib-tests;
        inherit distrox-gui;

        distrox-gui-clippy = craneLib.cargoClippy {
          pname = "distrox-gui";
          inherit src;
          inherit nativeBuildInputs;
          buildInputs = guiBuildInputs;

          cargoArtifacts = distroxGuiArtifacts;
          cargoExtraArgs = "-p distrox-gui --all-features";
          cargoClippyExtraArgs = "-- --deny warnings";
        };

        distrox-fmt = craneLib.cargoFmt {
          inherit src;
          inherit nativeBuildInputs;
          buildInputs = guiBuildInputs;
        };
      };

      packages = {
        default = packages.distrox-gui;

        inherit distrox-lib;
        inherit distrox-lib-tests;
        inherit distrox-gui;
      };

      apps = {
        default = apps.distrox-gui;

        distrox-gui = flake-utils.lib.mkApp {
          name = "distrox-gui";
          drv = distrox-gui;
        };

        distrox-cli = flake-utils.lib.mkApp {
          name = "distrox-cli";
          drv = distrox-gui;
        };
      };

      devShells = {
        distrox = pkgs.mkShell {
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath guiBuildInputs;
          buildInputs = nativeBuildInputs ++ guiBuildInputs;

          nativeBuildInputs = nativeBuildInputs ++ [
            rustTarget
            unstable.cargo-tauri

            pkgs.wasm-bindgen-cli
            pkgs.cargo-msrv
            pkgs.cargo-deny
            pkgs.cargo-expand
            pkgs.cargo-bloat
            pkgs.cargo-fuzz
            pkgs.cargo-outdated
            pkgs.trunk

            pkgs.gitlint
            cargo-changelog.packages."${system}".changelog
          ];
        };

        default = devShells.distrox;
      };
    }
    );
}

