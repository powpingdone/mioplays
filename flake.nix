{
  description = "A very basic flake";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
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
          libPath = with pkgs;
            lib.makeLibraryPath [
              libGL
              libxkbcommon
              wayland
            ];
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
                wayland-protocols
                wayland
                fontconfig
              ]);
            buildInputs = packages;
            LD_LIBRARY_PATH = libPath;
            FONTCONFIG_FILE = pkgs.makeFontsConf {fontDirectories = [pkgs.noto-fonts.all];};
          };
        }
      );
  };
}
