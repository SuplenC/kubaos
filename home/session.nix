# The running session after a switch. A switch points ~/.config's links at new
# store files and writes none, so the programs that read KubaOS's defaults once
# keep the old ones, and Hyprland, which only notices a file written in place,
# misses the change too.
#
# Each generation records, in `kubaos-session` beside its `activate`, the
# store paths every such part starts from: its defaults in config/, and the
# package where KubaOS pins it. Activation compares the old generation's
# record with the new one's and hands what changed to `scripts/dot-session`,
# which reloads or restarts it in the user's running Hyprland, or does nothing
# when there is none. The Kotodama Daemon needs nothing here: it follows links
# into the store and is a systemd unit, which the switch restarts when it
# changes.
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  defaults =
    name: path:
    builtins.path {
      inherit path;
      name = "kubaos-session-${name}";
    };

  # What Hyprland's own config loads: the Lua files, not the two hypr tools'
  # files beside them.
  hyprlandConfig = builtins.path {
    path = ../config/hypr;
    name = "kubaos-session-hyprland";
    filter =
      path: _:
      !(lib.elem (baseNameOf path) [
        "hypridle.conf"
        "hyprlock.conf"
        "README.md"
      ]);
  };

  # The names are the parts `scripts/dot-session` knows. hyprlock is left out:
  # it starts afresh at every lock.
  parts = {
    hyprland = [ hyprlandConfig ];
    # The package carries the Kotodama QML module the Command menu loads.
    quickshell = [
      (defaults "quickshell" ../config/quickshell)
      kubaosPkgs.quickshell
    ];
    hypridle = [ (defaults "hypridle" ../config/hypr/hypridle.conf) ];
    ghostty = [ (defaults "ghostty" ../config/ghostty) ];
    tmux = [ (defaults "tmux" ../config/tmux) ];
  };

  record = pkgs.writeText "kubaos-session" (
    lib.concatLines (
      lib.mapAttrsToList (name: paths: "${name} ${lib.concatMapStringsSep " " toString paths}") parts
    )
  );

  # Activation's PATH holds home-manager's own tools only, not the caller's:
  # hyprctl and tmux come from the system or the new home.
  path = "${lib.makeBinPath [ pkgs.procps ]}:$PATH:/usr/local/bin:$newGenPath/home-path/bin:/usr/bin";
in
{
  home.extraBuilderCommands = ''
    ln -s ${record} $out/kubaos-session
  '';

  # A first activation (the installer's, or a new machine's) has no older
  # generation and no session started from one: it does nothing. It runs once
  # the user units (the Kotodama Daemon the shell's menu talks to) and the
  # user layer are in place, so what it restarts starts on the new ones.
  home.activation.reloadSession = lib.hm.dag.entryAfter [ "reloadSystemd" "userLayer" ] ''
    if [[ -v oldGenPath ]]; then
      run env PATH="${path}" \
        XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR:-/run/user/$(id -u)}" \
        ${pkgs.bash}/bin/bash ${../scripts/dot-session} apply "$oldGenPath" "$newGenPath" || true
    fi
  '';
}
