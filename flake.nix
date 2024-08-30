{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      crane-lib = (crane.mkLib pkgs).overrideToolchain (
        p:
          p.rust-bin.stable.latest.default
      );

      nativeBuildInputsCommon = with pkgs; [pkg-config];
      buildInputsCommon = [];

      cargo-meta = crane-lib.crateNameFromCargoToml {cargoToml = ./Cargo.toml;};
      commonArgs = {
        inherit (cargo-meta) pname version;
        src = crane-lib.cleanCargoSource (crane-lib.path ./.);
        strictDeps = true;

        buildInputs = buildInputsCommon;
        nativeBuildInputs = nativeBuildInputsCommon ++ [pkgs.makeWrapper];
      };
      cargoArtifacts = crane-lib.buildDepsOnly commonArgs;

      crate = crane-lib.buildPackage ({
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath commonArgs.buildInputs;

          postInstall = ''
            wrapProgram "$out/bin/$pname" --set LD_LIBRARY_PATH $LD_LIBRARY_PATH
          '';
        }
        // commonArgs
        // {inherit cargoArtifacts;});
    in {
      checks = {
        inherit crate;
        crate-clippy = crane-lib.cargoClippy (commonArgs // {inherit cargoArtifacts;});
      };

      packages.default = crate;

      apps.default = flake-utils.lib.mkApp {
        drv = crate;
      };

      devShells.default = crane-lib.devShell rec {
        checks = self.checks.${system};
        packages = buildInputsCommon ++ nativeBuildInputsCommon;
        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
      };
    });
}
