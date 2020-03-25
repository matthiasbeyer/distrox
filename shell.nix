{ pkgs ? (import <nixpkgs> {}) }:

let
  env = with pkgs.rustChannels.stable; [
    rust
    cargo
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
  ];
in

pkgs.mkShell rec {
    buildInputs     = env ++ dependencies;
    LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
}

