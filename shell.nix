{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
  at-spi2-core.dev
  cargo
  libevdev
  llvmPackages_latest.clang
  mold
  llvmPackages_latest.libclang.dev
  pkg-config
  rustc
  speechd
  xorg.libX11.dev
  xorg.libXi.dev
  xorg.libXtst
  ];

  # Environment variables
  ODILIA_LOG = "debug";
  RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=mold";
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
    BINDGEN_EXTRA_CLANG_ARGS = 
    (builtins.map (a: ''-I"${a}/include"'') (with pkgs; [
      glibc.dev
      llvmPackages_latest.clang
      speechd
    ]));
}
