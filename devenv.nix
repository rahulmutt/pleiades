{ pkgs, ... }:
let
  # Stock pkgs.astrolog 7.70 aborts under fortify/stackprotector on modern
  # glibc (buffer-overflow -> stack-smashing -> segfault).  Disable all
  # hardening so the CLI can run headless for cross-check cusp emission.
  # This derivation is verification-only and is never distributed.
  astrolog-patched = pkgs.astrolog.overrideAttrs (old: {
    hardeningDisable = [ "all" ];
  });
in {
  packages = [ pkgs.clang pkgs.libclang astrolog-patched ];
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  enterShell = ''
    echo "astrolog (patched): $(command -v astrolog || echo 'not-found')"
  '';
}
