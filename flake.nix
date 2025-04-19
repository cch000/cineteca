{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = {nixpkgs, ...}: let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in {
    devShells.${pkgs.system}.default = pkgs.mkShell {
      buildInputs = [
        pkgs.ffmpeg
      ];
      nativeBuildInputs = with pkgs; [
        pkg-config
        rustPlatform.bindgenHook
      ];
      packages = with pkgs; [
        nil
        alejandra
        rustc
        cargo
        clippy
        rust-analyzer
        rustfmt
      ];
    };
  };
}
