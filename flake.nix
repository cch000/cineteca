{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = {nixpkgs, ...}: let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in {
    devShells.${pkgs.system}.default = pkgs.mkShell {
      packages = with pkgs; [
        nil
        rustc
        cargo
        clippy
        rust-analyzer
        rustfmt
      ];
    };
  };
}
