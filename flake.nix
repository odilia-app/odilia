{
  description = "Odilia";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
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
              cargo
            ];

            shellHook = ''
              alias ls=eza
              alias find=fd
            '';
          };
      }
    );
}
