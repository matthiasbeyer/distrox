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

  gtk = with pkgs; [
    glib
    pango
    gdk-pixbuf
    atk
    gtk3

    libsoup
    webkitgtk
  ];
in

pkgs.mkShell rec {
    buildInputs     = env ++ dependencies ++ gtk;
    LIBCLANG_PATH   = "${pkgs.llvmPackages.libclang}/lib";
}

