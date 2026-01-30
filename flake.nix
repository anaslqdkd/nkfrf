{
	description = "Notification Center Hyprland";

	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
	};

	outputs = { self, nixpkgs }:
		let
		system = "x86_64-linux";
	pkgs = import nixpkgs { inherit system; };
	deps = [
		pkgs.gtk4
			pkgs.cargo
			pkgs.rustc
			pkgs.pkg-config
			pkgs.libadwaita
			pkgs.gobject-introspection
			pkgs.gtk4-layer-shell
	];

	typelibPath = "${pkgs.gtk4}/lib/girepository-1.0:" +
		"${pkgs.libadwaita}/lib/girepository-1.0:" +
		"${pkgs.gobject-introspection}/lib/girepository-1.0";
	in
	{
		devShells.${system}.default = pkgs.mkShell {
			buildInputs = deps;

			shellHook = ''
				export GI_TYPELIB_PATH=${typelibPath}:$GI_TYPELIB_PATH
				export LD_LIBRARY_PATH=/home/ash/parser
				'';
		};
	};
}

