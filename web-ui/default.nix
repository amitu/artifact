
with import <nixpkgs> {};

stdenv.mkDerivation {
 name = "elm-example";
 src = ./.;
 buildInputs = [ elmPackages.elm nodejs ];

 patchPhase = ''
   patchShebangs node_modules/webpack
 '';
 buildPhase = ''
   npm run build
 '';

 installPhase = ''
   mkdir $out
   cp dist/* $out/
 '';
}
