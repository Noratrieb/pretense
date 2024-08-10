{ pkgs ? import <nixpkgs> { }, ... }: pkgs.rustPlatform.buildRustPackage {
  src = pkgs.lib.cleanSource ./.;
  pname = "pretense";
  version = "0.1.0";
  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    mainProgram = "pretense";
  };
}
