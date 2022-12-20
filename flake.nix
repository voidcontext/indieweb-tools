{
  inputs.nru.url = "github:voidcontext/nix-rust-utils?refs/heads/tags=v0.1.1+rust-1.66.0";

  outputs = { nru, ...}: 
    nru.lib.mkOutputs {
      src = ./.;
      pname = "indieweb-tools";
      version = "0.1.0";
    };
}
