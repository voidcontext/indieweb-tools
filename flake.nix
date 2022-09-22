{
  # Use the current stable release of nixpkgs
  inputs.nixpkgs.url = "nixpkgs/release-22.05";

  # flake-utils helps removing some boilerplate
  inputs.flake-utils.url = "github:numtide/flake-utils";

  # Oxalica's rust overlay to easyly add rust build targets using rustup from nix
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  inputs.crane.url = "github:ipetkov/crane";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { self, crane, ... }@inputs: inputs.flake-utils.lib.eachDefaultSystem (system:
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
          
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rust;
      
      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        
        inherit buildInputs nativeBuildInputs;
      };
      
      
      indieweb-tools-deps = craneLib.buildDepsOnly (commonArgs // {
        pname = "indieweb-tools";
      });
      
      indieweb-tools = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
        pname = "indieweb-tools";
        version = "0.1.0";
        cargoArtifacts = indieweb-tools-deps;
        
        inherit buildInputs nativeBuildInputs;
      };
          
      mkApp = name: {
        "${name}" = {
          type = "app";
          program = "${indieweb-tools}/bin/${name}";
        };
      };
    in
    rec {
    
      checks = {
        inherit indieweb-tools;
      };
      
      apps = (mkApp "orion") // (mkApp "app-auth") // (mkApp "janitor");
    
      packages.default = indieweb-tools;
      packages.indieweb-tools-deps = indieweb-tools-deps;

      devShells.default = pkgs.mkShell {
        buildInputs = nativeBuildInputs ++ buildInputs ++ [
          rust
          pkgs.cargo-outdated
          pkgs.cargo-watch
          pkgs.rust-analyzer
          pkgs.rustfmt
          pkgs.nixpkgs-fmt
        ];
      };
    }
  );
}