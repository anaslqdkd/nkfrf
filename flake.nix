{
  description = "Notification Center Hyprland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };

    rustPlatform = pkgs.rustPlatform;

    deps = with pkgs; [
      gtk4
      libadwaita
      gtk4-layer-shell
      gobject-introspection
      pkg-config
    ];

    typelibPath = pkgs.lib.makeSearchPath "lib/girepository-1.0" [
      pkgs.gtk4
      pkgs.libadwaita
      pkgs.gobject-introspection
    ];
  in
  {
    packages.${system}.default = rustPlatform.buildRustPackage {
      pname = "nkrfk";
      version = "0.1.0";

      src = self;

      cargoLock = {
        lockFile = ./Cargo.lock;
      };

      nativeBuildInputs = [
        pkgs.pkg-config
		 pkgs.makeWrapper
      ];

      buildInputs = deps;

      postInstall = ''
        wrapProgram $out/bin/nkfrf \
          --set GI_TYPELIB_PATH ${typelibPath}
      '';
    };

    devShells.${system}.default = pkgs.mkShell {
      buildInputs = deps ++ [
        pkgs.cargo
        pkgs.rustc
      ];

      shellHook = ''
        export GI_TYPELIB_PATH=${typelibPath}:$GI_TYPELIB_PATH
      '';
    };
  };
}

