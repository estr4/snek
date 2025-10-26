{
  description = "Rust Development Environment";
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
    env = pkgs.mkShell.override {stdenv = pkgs.clangStdenv;} {
      packages = with pkgs; [
        glfw
        cmake
        clang
        wayland
        # Web support (uncomment to enable) -- Untested - @JamesKEbert
        # emscripten
      ];
        
      LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
        libGL
        xorg.libXrandr
        xorg.libXinerama
        xorg.libXcursor
        xorg.libXi
      ];
      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    };
    
  in {
    devShells.x86_64-linux.default = env;
  };
}
