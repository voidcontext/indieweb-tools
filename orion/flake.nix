{
  # Use the current stable release of nixpkgs
  inputs.nixpkgs.url = "nixpkgs/release-22.05";

  # flake-utils helps removing some boilerplate
  inputs.flake-utils.url = "github:numtide/flake-utils";

  # Oxalica's rust overlay to easyly add rust build targets using rustup from nix
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  inputs.nix-utils.url = "git+https://github.com/voidcontext/nix-utils";
  inputs.nix-utils.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nix-utils.inputs.rust-overlay.follows = "rust-overlay";

  outputs = { self, nix-utils, ... }@inputs: inputs.flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ inputs.rust-overlay.overlays.default ];

      pkgs = import inputs.nixpkgs { inherit system overlays; };

      rust = pkgs.rust-bin.stable."1.61.0".default;

      nativeBuildInputs = with pkgs.lib;
        (optional pkgs.stdenv.isLinux pkgs.pkg-config);

      buildInputs = with pkgs.lib;
        (optional pkgs.stdenv.isLinux pkgs.openssl) ++
        (optional (system == "x86_64-darwin")
          pkgs.darwin.apple_sdk.frameworks.Security);

      orion = nix-utils.rust.${system}.mkRustBinary pkgs {
        src = ./.;
        inherit rust nativeBuildInputs buildInputs;
      };
    in
    rec {
      packages.default = orion;
      checks.default = orion;

      devShells.default = pkgs.mkShell {
        buildInputs = nativeBuildInputs ++ buildInputs ++ [
          rust
          pkgs.cargo-outdated
          pkgs.rust-analyzer
          pkgs.rustfmt
          pkgs.nixpkgs-fmt
        ];
      };
    }
  );
}
