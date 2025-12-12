{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };

        naersk' = pkgs.callPackage naersk { };

        buildInputs = with pkgs; [ ];

        nativeBuildInputs =
          with pkgs;
          [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "cargo"
                "rustc"
              ];
            })

            pkg-config
          ]
          # TODO(mdeand): Add support for macOS
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            alsa-lib
            vulkan-loader
          ];
      in
      rec {
        defaultPackage = packages.gpu-compositor-experiment;
        packages = {
          gpu-compositor-experiment = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
          };
        };

        devShell = pkgs.mkShell {
          RUSTFLAGS = "-Clink-args=-Wl,-rpath,${pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs)}";

          RUST_SRC_PATH = "${
            pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
            }
          }/lib/rustlib/src/rust/library";

          nativeBuildInputs =
            with pkgs;
            [
              nixfmt
              rustc
              rustfmt
              cargo
              clippy
              rust-analyzer
            ]
            ++ buildInputs
            ++ nativeBuildInputs;
        };
      }
    );
}
