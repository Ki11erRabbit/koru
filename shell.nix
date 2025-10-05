{ pkgs ? import <nixpkgs> {} }: 

pkgs.mkShell {
  buildInputs = [
    pkgs.llvmPackages.libclang.lib
    pkgs.guile
    pkgs.pkg-config
    pkgs.llvmPackages.clang
  ];

  shellHook = ''
    export GUILE_DIR=${pkgs.guile.dev}/lib
    export LIBCLANG_PATH=${pkgs.llvmPackages.libclang.lib}/lib
  '';
}
