{
  description = "A very basic flake";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
  };

  outputs = {
    nixpkgs,
    fenix,
    ...
  }: let
    lib = nixpkgs.lib;
  in {
    devShells =
      lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ] (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [fenix.overlays.default];
          };
        in {
          default = pkgs.mkShell rec {
            packages =
              [
                fenix.packages.${system}.stable.toolchain
              ]
              ++ (with pkgs; [
                rust-analyzer
                cargo-expand
                slint-lsp
                slint-viewer
              ]);
            buildInputs = packages;
          };
        }
      );
  };
}

