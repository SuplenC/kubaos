# The file manager: Files (Nautilus, from apt) by default, with yazi a key
# away through kubaos-files (crates/kubaos-files).
#
# - Nautilus is the default for folders and archives: `xdg-open <dir>` and
#   every app's "Open folder" open it there, and double-clicking an archive
#   extracts it next to itself, as Nautilus does for the archive types it is
#   the default app for.
# - org.freedesktop.FileManager1 ("Show in folder" in Chrome's downloads, VS
#   Code) is Nautilus's own D-Bus service in /usr/share; KubaOS ships no
#   service file of its own, which would shadow it from ~/.local/share.
# - yazi keeps a desktop entry, so it is under "Open With" for any folder;
#   ripdrag drags files out of it (user-layer/yazi/keymap.toml has the key).
# - The right-click actions are nautilus-python extensions: Ghostty's "Open in
#   Ghostty" and KubaOS's own, `home/files/kubaos-nautilus.py`.
# - A daily timer tidies ~/Downloads when the setting is on (`kubaos-files
#   tidy on`); with it off, the default, the timer's run does nothing.
#
# The files go under ~/.local/share, which xdg-open, GLib and nautilus-python
# always read: ~/.nix-profile/share is only on XDG_DATA_DIRS once a login
# shell has run.
{
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  files = lib.getExe kubaosPkgs.kubaos-files;
  nautilus = "org.gnome.Nautilus.desktop";
  # Archive types Nautilus's desktop entry lists, so it extracts them.
  archives = [
    "application/zip"
    "application/x-7z-compressed"
    "application/vnd.rar"
    "application/x-tar"
    "application/x-compressed-tar"
    "application/x-bzip-compressed-tar"
    "application/x-bzip2-compressed-tar"
    "application/x-xz-compressed-tar"
    "application/x-zstd-compressed-tar"
    "application/gzip"
    "application/x-xz"
    "application/zstd"
  ];
  images = [
    "image/jpeg"
    "image/png"
    "image/gif"
    "image/webp"
    "image/avif"
    "image/heic"
    "image/jxl"
    "image/bmp"
    "image/tiff"
    "image/svg+xml"
  ];
  videos = [
    "video/mp4"
    "video/webm"
    "video/x-matroska"
    "video/quicktime"
    "video/x-msvideo"
    "video/mpeg"
    "video/ogg"
  ];
  defaults = {
    "inode/directory" = "${nautilus};kubaos-files.desktop";
    "application/pdf" = "org.gnome.Papers.desktop";
  }
  // lib.genAttrs archives (_: nautilus)
  // lib.genAttrs images (_: "org.gnome.Loupe.desktop")
  // lib.genAttrs videos (_: "org.gnome.Showtime.desktop");
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
      Name=Yazi
      GenericName=File manager
      Comment=Your files in yazi, in a terminal window
      Icon=system-file-manager
      Exec=${files} open %U
      Terminal=false
      MimeType=inode/directory;
      Categories=System;FileTools;FileManager;
      Keywords=yazi;folder;directory;explorer;
    '';

    # KubaOS's defaults, not the user's choice: ~/.config/mimeapps.list comes
    # first in the lookup, so a default set there (`xdg-mime default`, Chrome
    # making itself the browser) wins, and stays writable. home-manager's
    # xdg.mimeApps would make that file a read-only link instead. It names
    # only apps KubaOS ships.
    # https://specifications.freedesktop.org/mime-apps-spec/latest/file.html
    "applications/mimeapps.list".text = lib.generators.toINI { } {
      "Default Applications" = defaults;
    };

    "nautilus-python/extensions/ghostty.py".source =
      "${pkgs.ghostty}/share/nautilus-python/extensions/ghostty.py";
    "nautilus-python/extensions/kubaos.py".source = ./files/kubaos-nautilus.py;
  };

  dconf.settings = {
    # Thumbnails on network shares and phones too, not only on local disks.
    "org/gnome/nautilus/preferences".show-image-thumbnails = "always";
    "org/gtk/gtk4/settings/file-chooser".sort-directories-first = true;
  };

  systemd.user.services.kubaos-tidy-downloads = {
    Unit.Description = "Tidy ~/Downloads into folders by kind, when the setting is on";
    Service = {
      Type = "oneshot";
      ExecStart = "${files} tidy --scheduled";
    };
  };
  systemd.user.timers.kubaos-tidy-downloads = {
    Unit.Description = "Tidy ~/Downloads daily, when the setting is on";
    Timer = {
      OnCalendar = "daily";
      # A laptop asleep at midnight tidies once it wakes.
      Persistent = true;
      RandomizedDelaySec = "1h";
    };
    Install.WantedBy = [ "timers.target" ];
  };

  # A running dbus-daemon forgets the FileManager1 service file older
  # generations linked under ~/.local/share only once it reloads, and until
  # then starts yazi for "Show in folder". No session bus (a switch from a
  # tty) is fine: the next one reads its folders afresh.
  home.activation.reloadSessionBus = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    run ${pkgs.dbus}/bin/dbus-send --session --type=method_call --dest=org.freedesktop.DBus \
      /org/freedesktop/DBus org.freedesktop.DBus.ReloadConfig 2>/dev/null || true
  '';
}
