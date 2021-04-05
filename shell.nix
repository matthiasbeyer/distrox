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
    libpsl
    openssl
    pkgconfig
    which
    zlib
    dbus
    libtool
    protobuf
  ];
in

pkgs.mkShell rec {
    buildInputs     = env ++ dependencies;
    LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
    PROTOC          = "${pkgs.protobuf}/bin/protoc";
}
