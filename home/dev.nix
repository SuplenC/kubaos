# Developer tools for everyone, and the user's own in ~/.local/bin on PATH.
# Rust's toolchain and cargo tools are the Rust Suite's (suites/rust.nix).
{ pkgs, ... }:
{
  home.packages = [ pkgs.hyperfine ];

  home.sessionPath = [ "$HOME/.local/bin" ];
}
