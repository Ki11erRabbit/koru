{ pkgs ? import <nixpkgs> {} }: 

pkgs.mkShell rec {
  buildInputs = [
    pkgs.llvmPackages.libclang.lib
    pkgs.guile
    pkgs.pkg-config
    pkgs.llvmPackages.clang
    pkgs.expat
    pkgs.fontconfig
    pkgs.freetype
    pkgs.freetype.dev
    pkgs.libGL
    pkgs.pkg-config
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    pkgs.wayland
    pkgs.libxkbcommon
  ];
  LD_LIBRARY_PATH =
    builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;

  shellHook = ''
    export GUILE_DIR=${pkgs.guile.dev}/lib
    export LIBCLANG_PATH=${pkgs.llvmPackages.libclang.lib}/lib
  '';
}
