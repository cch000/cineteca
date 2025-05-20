{
  description = "A TUI application for movies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    platforms = nixpkgs.lib.platforms.all;
    forAllplatforms = nixpkgs.lib.genAttrs platforms;

    name = "movies-tui";

    mkInputs = pkgs: {
      nativeBuildInputs = with pkgs; [
        pkg-config
        clang
        rustPlatform.bindgenHook
        clippy
      ];

      buildInputs = with pkgs; [
        ffmpeg
      ];
    };
  in {
    packages = forAllplatforms (
      platform: let
        pkgs = nixpkgs.legacyPackages.${platform};
        inputs = mkInputs pkgs;

        movies-tui = pkgs.rustPlatform.buildRustPackage {
          inherit (inputs) nativeBuildInputs buildInputs;
          inherit name;

          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          meta = with pkgs.lib; {
            inherit platforms;
            description = "TUI application for movies";
            maintainers = ["cch000"];
            license = licenses.gpl3Plus;
          };
        };
      in {
        inherit movies-tui;
        default = movies-tui;
      }
    );

    devShells = forAllplatforms (
      platform: let
        pkgs = nixpkgs.legacyPackages.${platform};
        inputs = mkInputs pkgs;
      in {
        default = pkgs.mkShell {
          inherit (inputs) nativeBuildInputs buildInputs;
          inputsFrom = [self.packages.${platform}.default];
        };
      }
    );
  };
}
