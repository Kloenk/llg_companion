with import <nixpkgs> {};
mkShell {
  buildInputs = [ pkgconfig openssl  ];
  shellHook = ''
    vscodium . &
    exec fish
  '';
}
