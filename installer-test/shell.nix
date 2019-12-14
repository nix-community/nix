let
  pkgs = import (builtins.fetchTarball { url = "channel:nixos-19.09"; }) {};
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    cloud-utils
    curl
    expect
    qemu
  ];
}
