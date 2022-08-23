{ pkgs ? import <nixpkgs> {} }:
let
  libs = with pkgs; [
  at-spi2-core.dev
  libevdev
  speechd
  xorg.libX11
  xorg.libXi.dev
  xorg.libXtst
  ];
in pkgs.mkShell {
nativeBuildInputs = with pkgs; [
  cargo
  llvmPackages_latest.clang
  mold
  llvmPackages_latest.libclang.dev
  pkg-config
  rustc
  rustfmt
];
buildInputs = libs;

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
    LD_LIBRARY_PATH = let paths = builtins.map (p: "${p}/lib") libs;
    in pkgs.lib.strings.concatStringsSep ":" paths;
}
