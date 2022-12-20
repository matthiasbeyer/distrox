{
  description = "distrox";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.05";
    unstable-nixpkgs.url = "nixpkgs/nixos-unstable";
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

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, unstable-nixpkgs, ... }:
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
          pkgconfig
          which
          zlib

          freetype
          expat
          protobuf
        ];

        buildInputs = (with pkgs; [
          alejandra
          appimagekit
          atk
          cairo
          dbus
          dbus.lib
          dprint
          gdk-pixbuf
          glib.out
          gobject-introspection
          gtk3
          harfbuzz
          libayatana-appindicator-gtk3
          libffi
          libsoup
          nodejs-16_x
          openssl.out
          pango
          pkg-config
          treefmt
          webkitgtk
          zlib
        ]);

        guiBuildInputs = buildInputs ++ (with pkgs.xorg; [
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
          inherit (tomlInfo) pname;
          inherit src;
          inherit nativeBuildInputs;
          inherit buildInputs;
          cargoExtraArgs = "-p distrox-lib --all-features";
        };

        distroxGuiFrontendArtifacts = craneLib.buildDepsOnly {
          pname = "distrox-gui-frontend";
          inherit src;

          doCheck = false;
          cargoExtraArgs = "--all-features -p distrox-gui-frontend --target wasm32-unknown-unknown";
        };

        distroxGuiArtifacts = craneLib.buildDepsOnly {
          inherit (tomlInfo) pname;
          inherit src;
          inherit nativeBuildInputs;
          buildInputs = guiBuildInputs;
          cargoExtraArgs = "-p distrox-gui --all-features";
        };

        distrox-lib = craneLib.buildPackage {
          inherit (tomlInfo) pname version;
          inherit src;
          inherit nativeBuildInputs;
          inherit buildInputs;
          cargoExtraArgs = "-p distrox-lib --all-features";
          cargoArtifacts = distroxLibArtifacts;
        };

        distrox-lib-tests = let
          testBuildInputs = buildInputs ++ [ pkgs.cmake pkgs.jq ];
        in craneLib.buildPackage rec {
          inherit (tomlInfo) pname;
          inherit src;
          inherit nativeBuildInputs;
          buildInputs = testBuildInputs;

          CARGO_PROFILE = "test";
          cargoExtraArgs = "-p distrox-lib --tests";
          doCheck = false;
          installPhaseCommand = ''
            TESTS=$(cargo test -p distrox-lib --no-run --message-format json-render-diagnostics | \
              jq -r 'select(.reason == "compiler-artifact" and .profile.test == true) | .executable')

            mkdir -p $out/bin/
            for test in $TESTS; do
              cp -v "$test" $out/bin/
            done
          '';
        };

        distrox-gui-frontend = craneLib.buildPackage {
          inherit (tomlInfo) version;
          inherit src;
          inherit nativeBuildInputs;
          pname = "distrox-gui-frontend";

          # Override crane's use of --workspace, which tries to build everything.
          cargoCheckCommand = "cargo check --release";
          cargoBuildCommand = "cargo build --release";
          cargoTestCommand = "cargo test --profile release --lib";

          doCheck = false;
          cargoArtifacts = distroxGuiFrontendArtifacts;
          cargoExtraArgs = "--all-features -p distrox-gui-frontend --target wasm32-unknown-unknown";
        };

        distrox-gui = craneLib.buildPackage {
          inherit (tomlInfo) pname version;
          inherit src;
          inherit nativeBuildInputs;

          preBuild = ''
            mkdir -p gui/frontend/dist
            ln -s ${distrox-gui-frontend}/bin/distrox-gui-frontend.wasm gui/frontend/dist/distrox-gui-frontend.wasm
          '';

          buildInputs = guiBuildInputs;
          cargoExtraArgs = "-p distrox-gui --all-features";
          cargoArtifacts = distroxGuiArtifacts;
        };
      in
      rec {
        checks = {
          inherit distrox-lib;
          inherit distrox-lib-tests;
          inherit distrox-gui;
          inherit distrox-gui-frontend;
          default = distrox-gui;

          distrox-gui-clippy = craneLib.cargoClippy {
            inherit (tomlInfo) pname;
            inherit src;
            inherit nativeBuildInputs;
            buildInputs = guiBuildInputs;

            preBuild = ''
              mkdir -p gui/frontend/dist
              ln -s ${distrox-gui-frontend}/bin/distrox-gui-frontend.wasm gui/frontend/dist/distrox-gui-frontend.wasm
            '';

            cargoArtifacts = distroxGuiArtifacts;
            cargoExtraArgs = "-p distrox-gui --all-features";
            cargoClippyExtraArgs = "-- --deny warnings";
          };

          distrox-fmt = craneLib.cargoFmt {
            inherit (tomlInfo) pname;
            inherit src;
            inherit nativeBuildInputs;
            buildInputs = guiBuildInputs;
          };
        };

        packages = {
          inherit distrox-lib;
          inherit distrox-lib-tests;
          inherit distrox-gui;
          inherit distrox-gui-frontend;
          default = packages.distrox-gui;
        };

        apps = {
          distrox-gui = flake-utils.lib.mkApp {
            name = "distrox-gui";
            drv = distrox-gui;
          };
          default = apps.distrox-gui;
        };

        devShells = {
          distrox = pkgs.mkShell {
            LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
            PROTOC          = "${pkgs.protobuf}/bin/protoc";
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath guiBuildInputs;

            XDG_DATA_DIRS = let
              base = pkgs.lib.concatMapStringsSep ":" (x: "${x}/share") [
                pkgs.gnome.adwaita-icon-theme
                pkgs.shared-mime-info
              ];

              gsettings_schema = pkgs.lib.concatMapStringsSep ":" (x: "${x}/share/gsettings-schemas/${x.name}") [
                pkgs.glib
                pkgs.gsettings-desktop-schemas
                pkgs.gtk3
              ];
            in "${base}:${gsettings_schema}";

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
            ];
          };

          default = devShells.distrox;
        };
      }
    );
}

