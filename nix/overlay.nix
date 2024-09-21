(final: prev: {
  wasm-bindgen-cli = prev.wasm-bindgen-cli.overrideAttrs (old: {
    version = "0.2.93";
    src = prev.fetchCrate {
      pname = "wasm-bindgen-cli";
      version = final.wasm-bindgen-cli.version;
      sha256 = "sha256-DDdu5mM3gneraM85pAepBXWn3TMofarVR4NbjMdz3r0=";
    };
    cargoDeps = old.cargoDeps.overrideAttrs (_: {
      src =  final.wasm-bindgen-cli.src; # You need to pass "src" here again,
                                         # otherwise the old "src" will be used.
      outputHash = "sha256-birrg+XABBHHKJxfTKAMSlmTVYLmnmqMDfRnmG6g/YQ=";
    });
  });
  trunk = prev.trunk.overrideAttrs (old: {
    version = "0.17.5";
    src = prev.fetchCrate {
      pname = "trunk";
      version = final.trunk.version;
      sha256 = "sha256-kOZmvpM13ccP1rAp4NFJigvhperlBTqOHYK/Y2p/nYk";
    };
    doCheck = false;
    cargoDeps = old.cargoDeps.overrideAttrs (_: {
      src =  final.trunk.src; # You need to pass "src" here again,
                              # otherwise the old "src" will be used.
      outputHash = "sha256-DxRf2JG6HsgqbK1In2HRaotel1UYj6dMtULTHk66/Ws=";
    });
  });
})

