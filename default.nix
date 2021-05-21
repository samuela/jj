### BROKEN

{ lib, rustPlatform, ... }:

{
  jj = rustPlatform.buildRustPackage rec {
    pname = "jj";
    version = "0.0.0";
    src = ".";
    # See https://stackoverflow.com/a/57230822/3880977.
    unpackPhase = "ls -al";
    cargoSha256 = lib.fakeSha256;
  };
}
