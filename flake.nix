{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    geng.url = "github:geng-engine/cargo-geng";
    geng.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = inputs: (
    let system = "x86_64-linux";
    in {
      devShells.${system}.default = inputs.geng.lib.mkShell {
        inherit system;
      };
    }
  );
}
