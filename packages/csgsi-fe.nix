{ src, rustPlatform, trunk-ng, wasm-bindgen-cli }:
rustPlatform.buildRustPackage {
  pname = "csgsi-fe";
  version = "0.1.0";

  inherit src;
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
