# See https://github.com/vantage-sh/ec2instances.info/blob/master/requirements.txt
let
  # Last updated: 2022-03-07. Check for new commits at status.nixos.org.
  pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/2c909d208d81fcad2c4e29f0c87c384f416176ba.tar.gz") { };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    python3
    python3Packages.invoke
    python3Packages.invocations
    python3Packages.boto
    python3Packages.Mako
    python3Packages.lxml
    python3Packages.requests
    python3Packages.six
    python3Packages.boto3
  ];
}
