{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }@inputs:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
      fenixLib = fenix.packages."x86_64-linux";
      rustToolchain = fenixLib.stable.withComponents [ "clippy" "rustc" "cargo" "rust-src" "rustfmt" "rust-analyzer" ];
      
      # Common dependencies
      xorgBuildInputs = with pkgs.xorg; [ libX11 libXcursor libXi libXrandr libXft libXinerama libXfixes];
      waylandBuildInputs = with pkgs; [ libxkbcommon wayland ];
      commonBuildInputs = with pkgs; [ 
        pango cairo glib
        systemd
        mesa
        egl-wayland
        vulkan-loader
        fontconfig
        fltk
        cmake
      ] ++ xorgBuildInputs ++ waylandBuildInputs;
    in {
      packages."x86_64-linux".default = pkgs.rustPlatform.buildRustPackage {
        pname = "quemoji";
        version = "0.1.0";
        
        src = ./.;
        
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
        
        nativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
          cmake
        ];
        
        buildInputs = commonBuildInputs;

        postFixup = ''
          patchelf --set-rpath "${pkgs.lib.makeLibraryPath commonBuildInputs}" $out/bin/quemoji
        '';
      };

      devShells."x86_64-linux".default = let
        nativeBuildInputs = with pkgs; [ rustToolchain pkg-config bacon ];
        buildInputs = commonBuildInputs;
      in pkgs.mkShell {
        inherit buildInputs nativeBuildInputs;
        shellHook = ''
          export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}
        '';
      };
    };
}
