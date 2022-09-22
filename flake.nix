{
  # Use the current stable release of nixpkgs
  # inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  # flake-utils helps removing some boilerplate
  # inputs.flake-utils.url = "github:numtide/flake-utils";

  # Oxalica's rust overlay to easyly add rust build targets using rustup from nix
  # inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  # inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  # inputs.nix-utils.url = "git+https://github.com/voidcontext/nix-utils";
  # inputs.nix-utils.inputs.nixpkgs.follows = "nixpkgs";
  # inputs.nix-utils.inputs.rust-overlay.follows = "rust-overlay";
  
  inputs.nci.url = "github:yusdacra/nix-cargo-integration/huge-refactor";
  # inputs.nci.inputs.nixpkgs.follows = "nixpkgs";
  # inputs.nci.inputs.rust-overlay.follows = "rust-overlay";
  
  outputs = { self, nci, ... }@inputs: 
    let
      buildInputs = pkgs: with pkgs.lib; 
        (optional (pkgs.stdenv.isDarwin) pkgs.darwin.apple_sdk.frameworks.Security);

      outputs = nci.lib.makeOutputs {
        root = ./.; 
        
        config = common: {
          cCompiler = 
            with common.pkgs;
              if stdenv.isLinux
              then gcc
              else clang;
        };
        # overrides = common: {
        #   orion =  {
        #   };
        # #   shell = prev: {
        # #     packages = prev.packages ++ (buildInputs prev.pkgs);
        # #   };
        # };
      };
    in
      outputs;
  # inputs.flake-utils.lib.eachDefaultSystem (system:
  #   let
  #     overlays = [ inputs.rust-overlay.overlays.default ];

  #     pkgs = import inputs.nixpkgs { inherit system overlays; };

  #     rust = pkgs.rust-bin.stable."1.61.0".default;

  #     nativeBuildInputs = with pkgs.lib;
  #       (optional pkgs.stdenv.isLinux pkgs.pkg-config);

  #     buildInputs = with pkgs.lib;
  #       (optional pkgs.stdenv.isLinux pkgs.openssl) ++
  #       (optional (system == "x86_64-darwin")
  #         pkgs.darwin.apple_sdk.frameworks.Security);
          
  #     mkCrate = crateRoot : nix-utils.rust.${system}.mkRustBinary pkgs {
  #       src = ./.;
  #       doCheck = false;
  #       cargoLock = ./Cargo.lock;
  #       postUnpack = ''
  #         cp ${./Cargo.lock} ''$sourceRoot/Cargo.lock
  #       '';
  #       inherit rust nativeBuildInputs buildInputs crateRoot;
  #     };

  #     orion = mkCrate "orion";
  #     app-auth = mkCrate "app-auth";
      
  #     cargo-tests = pkgs.stdenv.mkDerivation {
  #       src = ./.;
  #       name = "iwt-cargo-tests";
  #       buildInputs = [rust];
  #       buildPhase = ''echo "Skipping builPhase"'';
  #       installPhase = ''mkdir -p $out'';
  #       checkPhase = ''
  #         cargo fmt --check
  #         cargo test
  #       '';
  #     };
      
  #     shared.commons = nix-utils.rust.${system}.mkRustBinary pkgs {
  #       src = ./shared/commons;
  #       inherit rust nativeBuildInputs buildInputs;
  #     };
  #   in
  #   rec {
  #     packages.app-auth = app-auth;
  #     packages.orion = orion;

  #     checks.app-auth = app-auth;
  #     checks.orion = orion;
  #     checks.cargo-tests = cargo-tests;
      

  #     devShells.default = pkgs.mkShell {
  #       buildInputs = nativeBuildInputs ++ buildInputs ++ [
  #         rust
  #         pkgs.cargo-outdated
  #         pkgs.cargo-watch
  #         pkgs.rust-analyzer
  #         pkgs.rustfmt
  #         pkgs.nixpkgs-fmt
  #       ];
  #     };
  #   }
  # );
}