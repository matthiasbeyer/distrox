{ ... }:

let
  moz_overlay = import (
    builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz
  );

  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  env = with pkgs; [
    rustChannels.stable.rust-std
    rustChannels.stable.rust
    rustChannels.stable.rustc
    rustChannels.stable.cargo
  ];

  dependencies = with pkgs; [
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
  xorgPackages = with pkgs.xorg; [
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

pkgs.mkShell rec {
    buildInputs     = env ++ dependencies ++ xorgPackages;
    LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
    PROTOC          = "${pkgs.protobuf}/bin/protoc";
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath xorgPackages;
}
