{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    {
      nixosModules.default = import ./nix/portproxy.nix { inherit self; };
      nixosModules.portproxy = self.nixosModules.default;
    } // utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = pkgs.rustPackages_1_97;
        naersk-lib = pkgs.callPackage naersk {
          inherit (rust) cargo rustc clippy;
        };
      in
      {
        packages.default = naersk-lib.buildPackage ./.;
        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.pre-commit rust.cargo rust.rustc rust.rustfmt rust.clippy ];
          RUST_SRC_PATH = rust.rustPlatform.rustLibSrc;
        };
      }
    );
}
