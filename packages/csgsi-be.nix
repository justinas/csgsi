{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "csgsi-be";
  version = "0.1.0";

  src = lib.cleanSourceWith {
    filter = name: _type:
      let
        baseName = baseNameOf (toString name);
      in
        ! (lib.hasSuffix ".nix" baseName);
    src = lib.cleanSource ../.;
  };
  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  cargoBuildFlags = "-p csgsi_be";
}
