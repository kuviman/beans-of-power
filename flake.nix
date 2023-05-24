# https://scvalex.net/posts/63/
{
  inputs = {
    naersk.url = "github:nmattia/naersk/master";
    # This must be the stable nixpkgs if you're running the app on a
    # stable NixOS install.  Mixing EGL library versions doesn't work.
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        name = "getting-farted-on";
        waylandDeps = with pkgs; [
          libxkbcommon
          wayland
        ];
        xorgDeps = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        libDeps = with pkgs; waylandDeps ++ xorgDeps ++ [
          alsa-lib
          udev
          libGL
          xorg.libxcb
        ];
        nativeBuildDeps = with pkgs; [ pkg-config ];
        buildDeps = with pkgs; libDeps ++ [ xorg.libxcb ];
        libPath = pkgs.lib.makeLibraryPath libDeps;
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          doCheck = true;
          pname = name;
          nativeBuildInputs = nativeBuildDeps ++ [ pkgs.makeWrapper ];
          buildInputs = buildDeps;
          postInstall = ''
            wrapProgram "$out/bin/${name}" --prefix LD_LIBRARY_PATH : "${libPath}"
          '';
        };

        defaultApp = utils.lib.mkApp {
          drv = self.defaultPackage."${system}";
        };

        devShell = with pkgs; mkShell {
          nativeBuildInputs = nativeBuildDeps;
          buildInputs = buildDeps ++ [
            cargo
            rustPackages.clippy
            rustfmt
            rust-analyzer
          ];
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPath}"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
