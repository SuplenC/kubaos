# The file manager: yazi in Ghostty, through kubaos-files (crates/kubaos-files).
#
# - The Files desktop entry is the default for folders, so `xdg-open <dir>`
#   and every app's "Open folder" open yazi there.
# - org.freedesktop.FileManager1 on the session bus answers apps' "Show in
#   folder" (Chrome's downloads, VS Code). D-Bus starts `kubaos-files serve`
#   on the first call, and it quits once the calls stop.
# - ripdrag drags files out of yazi; the key is in the user layer's
#   yazi/keymap.toml (user-layer/yazi/keymap.toml seeds it).
#
# All three files go under ~/.local/share, which xdg-open, GLib and both D-Bus
# daemons always read: ~/.nix-profile/share is only on XDG_DATA_DIRS once a
# login shell has run, and dbus-daemon reads its service folders when the
# user manager starts it, before that.
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  files = lib.getExe kubaosPkgs.kubaos-files;
in
{
  home.packages = [
    kubaosPkgs.kubaos-files
    pkgs.ripdrag
  ];

  xdg.dataFile = {
    "applications/kubaos-files.desktop".text = ''
      [Desktop Entry]
      Type=Application
      Name=Files
      GenericName=File manager
      Comment=Your files in yazi
      Icon=system-file-manager
      Exec=${files} open %U
      Terminal=false
      MimeType=inode/directory;
      Categories=System;FileTools;FileManager;
      Keywords=yazi;folder;directory;explorer;
    '';

    # KubaOS's default, not the user's choice: ~/.config/mimeapps.list comes
    # first in the lookup, so a default set there (`xdg-mime default`, Chrome
    # making itself the browser) wins, and stays writable. home-manager's
    # xdg.mimeApps would make that file a read-only link instead.
    # https://specifications.freedesktop.org/mime-apps-spec/latest/file.html
    "applications/mimeapps.list".text = ''
      [Default Applications]
      inode/directory=kubaos-files.desktop
    '';

    "dbus-1/services/org.freedesktop.FileManager1.service".text = ''
      [D-BUS Service]
      Name=org.freedesktop.FileManager1
      Exec=${files} serve
    '';
  };

  # A running dbus-daemon finds a service file in a folder it didn't watch
  # only after it reloads; without one, "Show in folder" waits for the next
  # login. No session bus (a switch from a tty) is fine: the next one reads it.
  home.activation.reloadSessionBus = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    run ${pkgs.dbus}/bin/dbus-send --session --type=method_call --dest=org.freedesktop.DBus \
      /org/freedesktop/DBus org.freedesktop.DBus.ReloadConfig 2>/dev/null || true
  '';
}
