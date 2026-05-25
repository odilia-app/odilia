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
      craneLib = crane.mkLib pkgs;
      # this was originally in the mkshell derivation, but because this flake is able to build both a shell and binary derivation, those sets are lifted one level
      nativeBuildInputs =
      with pkgs;
      [
        rust-bin.stable.latest.default
        at-spi2-core.dev
        libevdev
        speechd
        libX11
        libXi.dev
        libXtst
      ];
      buildInputs =
      with pkgs;
      [
        pkg-config
        cargo
        rust-bin.stable.latest.default
      ];
      #this is going to purge the tree of any build artifacts, so only the source code remains
      src = craneLib.cleanCargoSource ./.;
      #because we're going to use this for both building the dependencies and the binary, this is cached instead of recomputed
      commonArgs = {
        inherit src buildInputs nativeBuildInputs;
      };
      
      #build all the dependencies first
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      # build the binary
      bin = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
      });
      in
      with pkgs;
      {
        packages = {
        inherit bin;
        default = bin;
          };
        devShells.default =
          mkShell {
            inputsFrom = [ bin ];
            shellHook = ''
              alias ls=eza
              alias find=fd
            '';
          };
      }
    );
}
