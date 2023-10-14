{ lib, rustPlatform, trunk-ng, wasm-bindgen-cli }:
rustPlatform.buildRustPackage {
  pname = "csgsi-fe";
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

  nativeBuildInputs = [ trunk-ng wasm-bindgen-cli ];
  buildPhase = ''
    export HOME=$(mktemp -d) # trunk puts stuff in $XDG_CACHE_HOME or ~/.cache
    wasm-bindgen --version
    trunk-ng build --offline --release csgsi_fe/index.html
  '';

  installPhase = ''
    mkdir -p $out/share/html
    cp -r csgsi_fe/dist/. $out/share/html
  '';
}
