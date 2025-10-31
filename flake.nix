{
  description = "A TUI program for movies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    systems = nixpkgs.lib.systems.flakeExposed;
    forAllsystems = nixpkgs.lib.genAttrs systems;

    name = "cineteca";

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
    packages = forAllsystems (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        inputs = mkInputs pkgs;

        cineteca = pkgs.rustPlatform.buildRustPackage {
          inherit (inputs) nativeBuildInputs buildInputs;
          inherit name;

          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          meta = with pkgs.lib; {
            inherit systems;
            description = "TUI application for movies";
            maintainers = ["cch000"];
            license = licenses.gpl3Plus;
          };
        };
      in {
        inherit cineteca;
        default = cineteca;
      }
    );

    formatter = forAllsystems (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in
        pkgs.writeShellApplication {
          name = "format";
          text = ''
            echo "Formatting Nix files..."
            find . -name "*.nix" -exec alejandra {} + > /dev/null 2>&1
            echo "Formatting Rust files..."
            find . -name "*.rs" -exec rustfmt {} +
          '';
        }
    );

    devShells = forAllsystems (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        inputs = mkInputs pkgs;
      in {
        default = pkgs.mkShell {
          inherit (inputs) nativeBuildInputs buildInputs;
          inputsFrom = [self.packages.${system}.default];
          packages = with pkgs; [
            nixd
            rustfmt
            alejandra
          ];
        };
      }
    );
  };
}
