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
            # Provide a complete Rust toolchain (rustc + cargo) from nixpkgs.
            # Without rustc here, `cargo build` inside `nix develop` falls through
            # to the GitHub runner's pre-installed rustup proxy at ~/.cargo/bin/cargo,
            # which on first use races to install a default toolchain into ~/.rustup.
            # That race occasionally produces concurrent rustup processes
            # ("info: downloading 6 components" twice), corrupting partial downloads
            # and leaving conflicting files (bin/rust-gdb, share/zsh) — surfacing as
            # E0463 "can't find crate for `core`" mid-build. Shipping rustc in the
            # devshell makes the toolchain self-contained and removes that path.
            buildInputs = [
              pkg-config
              cargo
              rustc
            ];

            shellHook = ''
              alias ls=eza
              alias find=fd
            '';
          };
      }
    );
}
