{ pkgs, ... }: {
  packages = [ pkgs.clang pkgs.libclang ];
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}
