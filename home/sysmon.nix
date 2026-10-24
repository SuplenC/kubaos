# The system monitor: kubaos-sysmon (crates/kubaos-sysmon), the service the
# bar's pill and the popup read a stream from (the shell starts it), and the
# command the menu changes its settings with. The popup's footer opens btop,
# which home/cli.nix installs.
{ kubaosPkgs, ... }:
{
  home.packages = [ kubaosPkgs.kubaos-sysmon ];
}
