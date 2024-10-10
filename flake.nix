{
  description = "A very basic flake";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { nixpkgs, rust-overlay, self, ... }:
    let
      inherit (pkgs) lib;
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };
      rustc = (pkgs.rust-bin.stable.latest.default.override {
        targets = [ "wasm32-unknown-unknown" ];
      });
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustc;
        inherit rustc;
      };
      src = lib.cleanSourceWith {
        filter = name: _type:
          let
            baseName = baseNameOf (toString name);
          in
            ! (lib.hasSuffix ".nix" baseName);
        src = lib.cleanSource ./.;
      };
      system = "x86_64-linux";
    in
    rec {
      devShells."${system}".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          trunk
        ];
      };

      checks."${system}" = rec {
        default = lint;
        lint = pkgs.stdenvNoCC.mkDerivation {
          name = "csgsi-lint";
          inherit src;

          # This replicates just enough of buildRustPackage,
          # i.e. the Cargo vendoring stuff, so that we can run clippy, etc.
          nativeBuildInputs = with rustPlatform; [ cargoSetupHook rustc ];
          cargoDeps = rustPlatform.importCargoLock {
            lockFile = ./Cargo.lock;
          };

          buildPhase = "";
          doCheck = true;
          checkPhase = ''
            cargo fmt --check
            cargo clippy
          '';
          installPhase = ''
            touch $out
            echo "All good."
          '';
        };
      };

      packages."${system}" = rec {
        csgsi-be = pkgs.callPackage ./packages/csgsi-be.nix { inherit rustPlatform src; };
        csgsi-fe = pkgs.callPackage ./packages/csgsi-fe.nix { inherit rustPlatform src; };
        default = pkgs.symlinkJoin {
          name = "csgsi";
          paths = [ csgsi-be csgsi-fe ];
        };
      };
    };
}
