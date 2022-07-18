{
  description = "HexoSynth";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
  flake-utils.lib.eachSystem ["x86_64-linux"] (system:
  {
    devShell =
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in
      pkgs.mkShell {
        buildInputs = with pkgs; [
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
          libglvnd
          libjack2
          openssl
          pkg-config
          python3Minimal
          qjackctl
          xorg.libX11
          xorg.libxcb
          xorg.libXcursor
        ];
        # this doesn't seem to do anything
        shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
          pkgs.xorg.libX11
          pkgs.xorg.libxcb
          pkgs.xorg.libXcursor
        ]}"'';
      };
  });
}
