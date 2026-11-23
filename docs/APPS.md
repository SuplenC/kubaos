# The Apps catalog's list

Every app KubaOS knows how to install, remove and keep updated is a folder under [`apps/`](../apps/), and the list is read from those folders whenever you ask for it, so it is never out of date and no file keeps it by hand. How the catalog works for you is in [`docs/USING.md`](USING.md), "Apps"; the folder's format in [`docs/CONTRACTS.md`](CONTRACTS.md), "The Apps catalog".

## Seeing it

```sh
kubaos app list                  # on a machine: the release's catalog by category, each app's summary, and which you picked
kubaos app list --markdown       # the same catalog as the table below, without your picks
just apps-list                   # in a checkout: the table, from that checkout's apps/
```

`kubaos app list --json` is the list for a program, in the same order (`docs/CONTRACTS.md`, "The Apps catalog").

## Icons

An app's icon is, in this order: Colloid's, the system's icon theme, named by the app; else the vendor's official logo or app icon as an `.svg` in its folder (a `.png` where the vendor publishes only a raster, its note in the PNG's comment), never redrawn or recoloured, placed on the catalog's rounded tile at the same visual weight as the others (white, or the vendor's own tile colour where its mark needs it), its header naming where it was taken from and that it is the vendor's trademark, used to identify the app; else a generated icon. Every app has one: the catalog refuses an app without.

A generated icon is a fixed picture, `apps/<id>/<id>.svg`, drawn once and committed:

```sh
just app-icon gamescope aspect-ratio   # the KubaOS Symbol of that Tabler name
just app-icon <id>                     # the app's initial, outlined
```

It is Colloid's plate in Colloid's own grey for a one-colour mark, `#4d4d4d` with a `#f2f2f2` glyph, the same on a dark menu and a light one, and its header names the command, so it can be drawn again. A vendor's icon, when one turns up, replaces it (`docs/CONTRACTS.md`, "An app's folder").

## Wayland

Every app with a Wayland backend comes up on it, never on XWayland (`docs/CONTRACTS.md`, "The Apps catalog", "Wayland"). The session's hints put almost all of them there. Spotify ignores them, so it gets a menu entry of KubaOS's, with the package's id, that starts it with `DISPLAY` unset. A removal takes the entry away. A menu entry of your own at `~/.local/share/applications/spotify.desktop` is never written over: Spotify's install stops and names it, and so does every update. Delete or rename your entry, then run `kubaos update`. Zoom's launcher starts it on XWayland unless its own settings say otherwise, so its install sets `xwayland=false` under `[General]` in `~/.config/zoomus.conf`. Every `kubaos update` sets it again, as it checks every step, so the only way back to XWayland is removing Zoom, which puts back the value you had, unless you changed it since. Which way Zoom captures your screen on Wayland is Zoom's own setting.

Tor Browser asks which you want, X11 or Wayland: `kubaos app add tor-browser --setup anonymity` or `--setup wayland`, and an Install Item for each in the Command menu. The Tor Project keeps Tor Browser on X11 by default, because it hasn't yet worked out what Wayland gives away to a site fingerprinting you, and a privacy default is yours to set, so KubaOS doesn't set one. X11, "Tor Browser", is the Tor Project's pick: your Tor Browser looks like everyone else's, and it's soft when the screen is scaled. Wayland, "Tor Browser on Wayland", is sharp at any scale. The X11 setup writes the launcher's two menu entries, with the package's ids, to start it with `MOZ_ENABLE_WAYLAND=0`, which Tor Browser reads before `GDK_BACKEND`; switching to Wayland or removing Tor Browser takes them away. A menu entry of your own at either path stops the X11 setup, as Spotify's does. `torbrowser-launcher` started from a terminal takes the terminal's environment, which is the session's: Wayland, unless you set `MOZ_ENABLE_WAYLAND=0` yourself.

These run on XWayland:

| App | Why |
|---|---|
| Ardour | Its toolkit, Ardour's own fork of GTK 2, draws on X11 alone. |
| Cryptomator | JavaFX, whose GTK launcher sets `GDK_BACKEND=x11`, has no Wayland backend. |
| GParted | It runs as root through pkexec, whose polkit action passes root only `DISPLAY` and `XAUTHORITY`, so it draws on XWayland's X server; root never gets your Wayland socket. |
| JetBrains Toolbox | The JetBrains Runtime's Wayland toolkit could run it (`--jvm-args` with `-Dawt.toolkit.name=WLToolkit`), but JetBrains doesn't support that for Toolbox, so it stays on X11. The IDEs it installs, and Android Studio, start on Wayland. |
| Minecraft Launcher | Mojang's launcher is X11 alone. |
| ONLYOFFICE | Its bundled Qt 5 has no Wayland plugin. |
| RawTherapee | Its AppImage sets `GDK_BACKEND=x11` as it starts, over the session's, for a crash its GTK had on Wayland. |
| Steam | Steam's client is X11 alone; its games run on XWayland or in gamescope. |
| VLC | VLC 3's Wayland window never worked properly; VideoLAN removed it in 3.0.24. Wayland output is VLC 4's. |
| Wine | Wine 11's Wayland driver is experimental, so it stays behind the X11 one, which wins while XWayland runs. `Graphics="wayland,x11"` under `HKCU\Software\Wine\Drivers` puts a prefix on it. |

The command-line tools, the services and the drivers open no window: they run in your terminal, behind a web page or in the kernel.

## The table's columns

One row an app, by category in the catalog's order (Browsers, Development, DevOps, AI, Communication, Media, Graphics, Office, Security, System, Games), then by id.

| Column | What it says |
|---|---|
| App | The app's name, and its id: what `kubaos app add` takes. |
| Category | Where the Command menu and the list group it. |
| Setups | The ways it can be set up, by the id `--setup` takes; none is the default. A dash: it comes one way. |
| Requires | The apps installed before it, each the way it installs on its own: the app's own, then `setup: apps` for those only one setup needs. |
| Summary | What it is, in one line of ours. |
| Source | Where its install takes it from, in the order KubaOS prefers them: an apt repo (the vendor's, named as in `system/packages.toml`, its key pinned), the Ubuntu archive, nixpkgs (through the User flake), a pinned .deb or a pinned download (an AppImage or an archive, checked against its sha256), a pinned DKMS module (a kernel module's source, checked against its sha256, which DKMS builds for each kernel), pinned container images (a stack of images each pinned by its digest, which a `run` step pulls), a web app (a website in a window of its own; a window on one of this machine's addresses opens what another source installed, so it isn't one). An app with setups says it for each setup, `setup: sources`, the app's own steps included. |
| Architectures | The machines it installs on, amd64 and arm64, in one setup at least, through every app it requires: an app that needs Chrome is amd64 only, as Chrome is. |
