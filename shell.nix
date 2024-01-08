with import <nixpkgs>{};
stdenv.mkDerivation {
  name = "env";
  nativeBuildInputs = [ cmake ninja ];
  buildInputs = [ 
    SDL2 
    SDL2.dev 
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    vulkan-loader
  ];
}
