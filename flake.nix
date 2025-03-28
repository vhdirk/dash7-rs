{
  description = "dash7-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }: {
    devShells = flake-utils.lib.eachDefaultSystemPassThrough (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
          };
        };
      in {
        "${system}".default = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              rust-bin.beta.latest.default
              cargo-binstall
              maturin
              uv

              systemd.dev
              pkg-config

              python3Packages.ipython
            ]
            ++ [
            ];
        };
      }
    );
  };
}
