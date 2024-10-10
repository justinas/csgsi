{ src, rustPlatform, binaryen, trunk, wasm-bindgen-cli, }:
rustPlatform.buildRustPackage {
  pname = "csgsi-fe";
  version = "0.1.0";

  inherit src;
  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  TRUNK_SKIP_VERSION_CHECK = "true";

  nativeBuildInputs = [ binaryen trunk wasm-bindgen-cli ];

  buildPhase = ''
    export HOME=$(mktemp -d) # trunk puts stuff in $XDG_CACHE_HOME or ~/.cache
    wasm-bindgen --version
    trunk build --offline --release csgsi_fe/index.html
  '';

  installPhase = ''
    mkdir -p $out/share/html
    cp -r csgsi_fe/dist/. $out/share/html
  '';
}
