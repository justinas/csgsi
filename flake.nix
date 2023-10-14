{
  description = "A very basic flake";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
  inputs.nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable-small";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { nixpkgs, nixpkgs-unstable, rust-overlay, ... }:
    let
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
      system = "x86_64-linux";
    in
    rec {
      devShells."${system}".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          trunk
        ];
      };

      # This check abuses `buildRustPackage` to set up cargo deps etc.,
      # then instead of actually building, just runs lint tools.
      # There might be a better way by manually using cargo hooks
      # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#hooks-hooks
      checks."${system}".default = packages."${system}".csgsi-be.overrideAttrs {
        buildPhase = ''
          touch $out
        '';
        checkPhase = ''
          cargo fmt --check
          #cargo clippy
        '';
        installPhase = ''
          echo "All good."
        '';
      };

      packages."${system}" = rec {
        csgsi-be = pkgs.callPackage ./packages/csgsi-be.nix { inherit rustPlatform; };
        csgsi-fe = pkgs.callPackage ./packages/csgsi-fe.nix { inherit rustPlatform; };
        default = pkgs.symlinkJoin {
          name = "csgsi";
          paths = [ csgsi-be csgsi-fe ];
        };
      };
    };
}
