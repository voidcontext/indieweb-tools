{
  inputs.nixpkgs.url = "nixpkgs/release-23.05";
  inputs.nixpkgs-unstable.url = "nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.nix-rust-utils.url = "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.8.1";
  inputs.nix-rust-utils.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nix-rust-utils.inputs.nixpkgs-unstable.follows = "nixpkgs-unstable";

  outputs = {
    nixpkgs,
    flake-utils,
    nix-rust-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      nru = nix-rust-utils.mkLib {inherit pkgs;};
      commonArgs = {
        src = ./.;
        buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
        ];
      };

      crate = nru.mkCrate (commonArgs
        // {
          doCheck = false;
        });
      checks = nru.mkChecks (commonArgs
        // {
          inherit crate;
          nextest = true;
        });
    in {
      inherit checks;

      packages.default = crate;

      devShells.default = nru.mkDevShell {
        inputsFrom = [crate];
        inherit checks;
      };
    });
}
