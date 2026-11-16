# The Apps catalog's list

Every app KubaOS knows how to install, remove and keep updated is a folder under [`apps/`](../apps/), and the list is read from those folders whenever you ask for it, so it is never out of date and no file keeps it by hand. How the catalog works for you is in [`docs/USING.md`](USING.md), "Apps"; the folder's format in [`docs/CONTRACTS.md`](CONTRACTS.md), "The Apps catalog".

## Seeing it

```sh
kubaos app list                  # on a machine: the release's catalog by category, each app's summary, and which you picked
kubaos app list --markdown       # the same catalog as the table below, without your picks
just apps-list                   # in a checkout: the table, from that checkout's apps/
```

`kubaos app list --json` is the list for a program, in the same order (`docs/CONTRACTS.md`, "The Apps catalog").

## The table's columns

One row an app, by category in the catalog's order (Browsers, Development, AI, Communication, Media, Graphics, Office, Security, System, Games), then by id.

| Column | What it says |
|---|---|
| App | The app's name, and its id: what `kubaos app add` takes. |
| Category | Where the Command menu and the list group it. |
| Setups | The ways it can be set up, by the id `--setup` takes; none is the default. A dash: it comes one way. |
| Requires | The apps installed before it, each the way it installs on its own: the app's own, then `setup: apps` for those only one setup needs. |
| Summary | What it is, in one line of ours. |
| Source | Where its install takes it from, in the order KubaOS prefers them: an apt repo (the vendor's, named as in `system/packages.toml`, its key pinned), the Ubuntu archive, nixpkgs (through the User flake), a pinned .deb or a pinned download (an AppImage or an archive, checked against its sha256), a web app (the website in a window of its own). An app with setups lists every setup's. |
| Architectures | The machines it installs on, amd64 and arm64, in one setup at least, through every app it requires: an app that needs Chrome is amd64 only, as Chrome is. |
