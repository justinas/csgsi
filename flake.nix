{
  description = "A very basic flake";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
  inputs.nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable-small";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { nixpkgs, nixpkgs-unstable, rust-overlay, self, ... }:
    let
      inherit (pkgs) lib;
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
          # TODO: remove after https://github.com/thedodd/trunk/pull/570 is mainlined
          # and available in nixos stable?
          (_final: _prev: {
            inherit (pkgsUnstable) trunk-ng;
            # Trunk is picky about the specific version of wasm-bindgen. Ugh.
            inherit (pkgsUnstable) wasm-bindgen-cli;
          })
        ];
      };
      pkgsUnstable = import nixpkgs-unstable {
        inherit system;
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

      # This replicates just enough of buildRustPackage,
      # i.e. the Cargo vendoring stuff, so that we can run clippy, etc.
      checks."${system}" = rec {
        default = lint;
        lint = pkgs.stdenvNoCC.mkDerivation {
          name = "csgsi-lint";
          inherit src;
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
