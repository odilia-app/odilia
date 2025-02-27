{
  description = "Odilia";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            nativeBuildInputs = [
              at-spi2-core.dev
              libevdev
              speechd
              xorg.libX11
              xorg.libXi.dev
              xorg.libXtst
            ];
            buildInputs = [
              pkg-config
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            ];

            shellHook = ''
              alias ls=eza
              alias find=fd
            '';
          };
      }
    );
}
