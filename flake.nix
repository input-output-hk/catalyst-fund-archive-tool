{
  inputs = { utils.url = "github:numtide/flake-utils"; };
  outputs = { self, nixpkgs, utils }:
    utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        cargoPackage =
          (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;

        overlay = self: super: {
          catalyst-fund-archive-tool = self.callPackage ({ lib, rustPlatform
            , fetchFromGitHub, pkg-config, openssl, protobuf, rustfmt }:
            rustPlatform.buildRustPackage rec {
              pname = cargoPackage.name;
              version = cargoPackage.version;
              src = ./.;
              cargoSha256 =
                "sha256-lhueTy1jR/pFXk7bAYH772X43wQS8VITVCtrLEa+9VY=";
              nativeBuildInputs = [ pkg-config rustfmt ];
              buildInputs = [ openssl ];
              configurePhase = ''
                cc=$CC
              '';
              doCheck = false;
              doInstallCheck = false;
            }) { };
        };
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ overlay ];
        };
      in {
        packages.catalyst-fund-archive-tool = pkgs.catalyst-fund-archive-tool;
        defaultPackage = pkgs.catalyst-fund-archive-tool;
        inherit overlay;
      });
}
