{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
    rust-nix.url = "github:input-output-hk/rust.nix/work";
    rust-nix.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, utils, rust-nix }:
  utils.lib.simpleFlake {
    inherit nixpkgs;
    systems = [ "x86_64-linux" "aarch64-linux" ];
    preOverlays = [ rust-nix ];
    overlay = final: prev:
    let lib = prev.lib;
    in {
      catalyst-fund-archive-tool = final.rust-nix.buildPackage {
        inherit ((builtins.fromTOML
        (builtins.readFile ./Cargo.toml)).package)
        name version;
        root = ./.;
        nativeBuildInputs = with final; [ pkg-config rustfmt ];
        buildInputs = with final; [ openssl ];
        #configurePhase = ''
        #      cc=$CC
        #'';
        doCheck = false;
        doInstallCheck = false;
      };
    };
    packages = { catalyst-fund-archive-tool }@pkgs: pkgs;
    devShell = { mkShell, rustc, cargo, pkg-config, openssl }:
    mkShell {
      buildInputs = [ rustc cargo pkg-config openssl ];
    };
  };
}
