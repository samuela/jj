{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
      rec {
        # `nix build`
        packages.jj = naersk-lib.buildPackage {
          pname = "jj";
          root = ./.;
        };
        defaultPackage = packages.jj;

        # `nix run`
        apps.jj = utils.lib.mkApp {
          drv = packages.jj;
        };
        defaultApp = apps.jj;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
        };
      });
}
