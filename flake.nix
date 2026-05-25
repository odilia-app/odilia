{
  description = "Odilia";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
    url = github:oxalica/rust-overlay;
    inputs = {
      nixpkgs.follows = "nixpkgs";
      flake-utils.follows = "flake-utils";
    };
    };
    crane = {
      url = "github:ipetkov/crane";
          inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
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
              rust-bin.stable.latest.default
              at-spi2-core.dev
              libevdev
              speechd
              libX11
              libXi.dev
              libXtst
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
              rust-bin.stable.latest.default
            ];

            shellHook = ''
              alias ls=eza
              alias find=fd
            '';
          };
      }
    );
}
