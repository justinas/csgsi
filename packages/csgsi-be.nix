{ src, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "csgsi-be";
  version = "0.1.0";

  inherit src;
  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  cargoBuildFlags = "-p csgsi_be";
}
