{
  description = "HexoSynth";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-21.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
  flake-utils.lib.eachSystem ["x86_64-linux"] (system:
  {
    devShell =
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      pkgs.mkShell {
        buildInputs = with pkgs; [
          cowsay
        ];
      };
  });
}
