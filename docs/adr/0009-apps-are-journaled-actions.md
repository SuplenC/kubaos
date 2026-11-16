# 9. An app is a folder of typed actions with inverses, journaled

Status: accepted (2026-11-16). Kuba's decisions in #435 (part of #93). Amends ADR 0008: an app is a folder of actions instead of a source plus hooks, and whether it is installed is read from its journal.

## Context

ADR 0008 made an app one source (apt, nixpkgs or a pinned `.deb`) plus an opaque pair of shell functions: `system_hooks` in `system/lib.sh` for root and `user_hooks` in `apps/user-hooks.sh` for the user. Installing an app is usually a process, though: a package, a group, a unit, a setting, a vendor tool's own installer. With hooks, nothing knows what an app changed. Each hook's cleanup is written by hand (`remove_t3_service` is a hundred lines of it), a failure halfway leaves whatever the hook got to, a removal has to guess what was there before (`t3-code-linger-was-off` is such a guess written to disk), and two apps can write the same file without either noticing.

## Decision

**An install is a list of typed actions, each with its inverse, recorded in a per-app journal. The journal is the box.** A failed install rolls back to where it started, and removing an app undoes exactly what its install did. Nobody writes cleanup by hand.

### The folder

`apps/<id>/app.toml` is the app; files its actions use sit beside it (templates, scripts a `run` calls, a web app's icon). Adding an app is one folder. The file says what the app is, then its actions in order:

```toml
name = "Docker"
category = "development"
tags = ["containers", "compose", "docker"]
summary = "Containers and Compose"
description = "Docker Engine is an open source containerization technology for building and containerizing your applications."
description_from = "https://docs.docker.com/engine/"
icon = "docker"

[[action]]
name = "Install Docker"
kind = "apt"
repo = "docker"
packages = ["docker-ce", "docker-ce-cli", "containerd.io"]

[[action]]
name = "Let you run docker without sudo"
kind = "group"
group = "docker"
```

Every app carries its description, all of it required and checked when the folder is read:

- `category`: one of `browsers`, `development`, `ai`, `communication`, `media`, `graphics`, `office`, `security`, `system`, `games`, which groups it with others (Kōbō's guided setup, `kubaos app list`). A new category is a decision for its PR, added here.
- `tags`: at least one word people search by ("browser", "containers", "passwords"), lowercase words, each once. They find the app while it isn't installed: they are keywords of its Install Item only, never of Remove.
- `summary`: one line of ours, in KubaOS's voice (`docs/BRAND.md`), saying what the app is.
- `description`: the app's own words, copied from its upstream (its AppStream metainfo, store listing or homepage) where the source's licence allows it, else a plain factual one; `description_from`, the `https://` URL it came from.

`icon` stays optional, as before.

An app with setups (ADR 0008) lists them as `[setup.<id>]` with `name` and `summary`, and each setup's own actions as `[[setup.<id>.action]]`. An install of a setup runs the app's `[[action]]`s, then the setup's, as one list: a step is its 1-based place in that list, so the actions every setup shares have the same steps in every setup. A `[[ready]]` (`check`, an argv run as the user in the app's folder with its words filled in, and `says`, the sentence when it fails), top level or in a setup, must pass before an install or a removal changes anything (T3 Code's "quit T3 Code first").

Strings take `@USER@`, `@HOME@`, `@ARCH@` and, in an action with a `version`, `@VERSION@`. A path is `~/…` (the home) or absolute; an absolute path an action writes is under `/etc/`, `/opt/` or `/usr/local/`. Templates (`from` with `template = true`) take the same words.

Two keys are reserved for what comes next, and refused until it does: `requires = ["<id>", { id = "<id>", min = "<version>" }]`, the apps installed first, in order, each with its own folder, actions and journal (a `min` the required app's source can't meet is refused before anything runs; removing a required app is refused, naming the app that needs it; a dependency installed only for another goes with the last app that needed it, unless the user picked it); and `stage = "configure"` on an action, which marks it part of the configure stage (`kubaos app configure <id>`), whose actions write the app's KubaOS defaults in the user layer, `~/.config/kubaos/<app>/`.

### The vocabulary

Every action has a `name` ("Download the t3 tool"), the step the user sees. Each kind knows how to **do** itself, **check** whether it is done, and **undo** itself, and runs as root or as the user by its kind and paths, never by the author's say, except `run`. An action resolved for a machine (its words filled in, the files of the folder it names read in) is what the journal records and what an undo runs.

| kind | fields | root? | do | check | undo |
|---|---|---|---|---|---|
| `apt` | `packages`, `repo`?, `arch`? | root | `system/apt-repo.sh <repo>`, then `apt-get install --no-install-recommends` of the missing packages, refused (nothing changed, the packages named) when `apt-get -s` says it would remove any package; the packages the simulation says it would newly install beyond the action's own (what they depend on that wasn't there) are journaled before anything is installed | dpkg has every package | `apt-get remove` the packages this action installed (configuration kept), refused (nothing removed, the packages named) when `apt-get -s` says it would take any other package with them; with them, the packages it pulled in that removing takes nothing else with (each simulated: one a package outside them depends on stays, and what it needs); never `apt-get autoremove`, which would take every package apt marked automatic; the repository stays, as in ADR 0008 |
| `deb` | `package`, `version`, `self_updates`, `[amd64]`/`[arm64]` `url` + `sha256` | root | download over https, refuse a sha256 that isn't the pin, `apt-get install` the file over whatever version is there, refused and its dependencies journaled as `apt`'s are | dpkg has the package (at the pin, unless `self_updates`) | `apt-get remove` it, if this action installed it, and what it pulled in, as `apt`'s undo does |
| `nix` | `packages` | user | the names into the User flake's `packages.toml` (a switch follows the run) | `packages.toml` has every name | the names this action added go |
| `download` | `to`, `version`, `unpack` (`tar-gz` or `file`), `strip`, `[amd64]`/`[arm64]` | by `to` | download, check the sha256, unpack into a staging folder beside `to` (a tarball with `--no-same-owner --no-same-permissions`, then no one but the owner may write anything in it: as root, everything is root's), move it into place (over an older pin's: that one moved aside, the new one renamed in, the old one removed) | `to` is there and, when the app's journal says its step put it there, from the pin the step recorded (the sha256, `unpack` and `strip`); one whose step a run stopped during is downloaded again | `to` goes, and the folders this action created, if empty |
| `file` | `to`, `from` (a file in the folder) or `text`, `template`, `mode` | by `to` | write it, creating missing folders | `to` has that content | `to` goes, and the folders this action created, if empty |
| `symlink` | `to`, `target` | by `to` | the link | `to` links to `target` | the link goes |
| `setting` | `file` (`.json` or `.toml`, under `~`), `key` (dotted), `value` | user | the key set in the file's own format and nothing else of it changed: TOML through its syntax tree, its comments, layout and types kept; JSON in place in its text, its indentation, spacing, key order and the text of its other values (a number as `1.50`) kept, and a table the key needs laid out as the file's own; a file that isn't settings of its format, a key under a value that isn't a table, or a key that is a table is refused, and the file is never rewritten | the key has `value` | while the file is exactly as the action left it, its old text back whole; else the key's old value back, or the key gone (with the tables above it the action made, once empty) if it had none, or the file gone if this action created it and nothing else is in it; a key the user has changed since, or a file no longer readable as settings, stays as it is |
| `unit` | `unit`, `scope` (`user` or `system`), `enable`?, `start`? | by scope | `systemctl enable`/`disable`, `start`/`stop`, after a `daemon-reload` | the unit is as asked: `start = true` is started, which is running or, for a oneshot (`Type=oneshot` without `RemainAfterExit`), run since the machine started and exited cleanly (`ActiveState` inactive, `Result` success, a main process exit since boot); any other service that is inactive isn't started, however cleanly it was stopped; `start = false` is not running | the unit back to how it was before, unless it was changed since |
| `group` | `group` | root | the group created if missing, the user who asked added to it | the user is in it | the user leaves it if this action added them; the group goes if this action created it |
| `linger` | | user | `loginctl enable-linger` | logind says the user lingers | `disable-linger`, if lingering was off before |
| `web_app` | `id`, `title` (one line, no control characters), `url` (`https://`, or `http://` on this machine: `localhost`, `127.0.0.1` or `[::1]` with its port, as Penpot's local stack serves), `icon` (an `.svg` in the folder) | user | a desktop entry, `~/.local/share/applications/kubaos-web-<id>.desktop` (`Name` the title, `Exec` `google-chrome --app=<url> --class=kubaos-web-<id>`, each argument quoted and its `%` doubled as the Desktop Entry spec asks, so a URL's `?`, `&`, `#` and `%20` reach the browser whole, `StartupWMClass` the same class, `Icon` the icon's path), and the icon, `~/.local/share/icons/hicolor/scalable/apps/kubaos-web-<id>.svg`: a web app in a window of its own, with a window class of its own | both files have that content | both go, and the folders this action created, if empty |
| `output` | `path` | by `path` | the folders above `path` that aren't there | `path` is there (found: a file an older install left, which the app never owned) | `path` goes, and the folders this action created, if empty: a file a program of the app writes later (Chrome's theme policy), which the app owns all the same, once the action made room for it |
| `run` | `do`, `check`, `undo` (argvs), `root`, `window`? | `root`, default no | `do` | `check` exits 0 | `undo` |

**`run` is the one escape hatch**, for vendor tools (`t3 service install`). Its `check` and `undo` are mandatory: the author says how to tell it is done and how to take it back, or there is no `run`. Its argvs are a program on the fixed `PATH`, `./<script>` in the app's folder, or, as the user only, one under the home (`@HOME@/.local/bin/t3`): root never runs a program the user could change; `@HOME@` and the rest are filled in, nothing else is. It runs as the user, with their environment and `HOME`, unless `root = true`: then as root, with root's own environment (`HOME=/root`, the fixed `PATH`) and nothing of the user's (no `XDG_*`, no `PYTHON*`, `NPM_*` or `CURL_*`). A `./<script>` is read into the action as it resolves, so the journal holds the scripts the step ran: a check, a resume and an undo run those, from a fresh private folder that is their working directory, whatever the app's folder holds by then or after the app left the catalog. A script reads nothing else of the folder; a file it needs is an action of its own (a `file`). With `window` (the sentence of the step left to the person), it opens a window, so it runs only in a run that may open one (`kubaos app add`); in one that may not (`kubaos update`), it is skipped, its sentence becomes a note, and the install is complete all the same. It is the only place an author's command runs.

**A download's pin is the journal's.** A file's content can be compared with its action, a download's can't: a tarball unpacked to a folder carries no sha256, and hashing a runtime on every run would be slow. So the journal says which pin a download is: its check is done only when the step that put `to` there recorded the pin the folder's action has now, and a release that changes the pin downloads it again, at a new path or over the old one. A `to` without `@VERSION@` is allowed, since a single program at a path others name (`~/.local/bin/tool`) has nowhere else to go; a folder a running program reads takes `@VERSION@`, so a new version lands beside the old one, which goes only once the new one stands (T3's runtime does).

The set is sized to the six apps and Penpot (whose hosted setup is a `web_app`) and stays small: a new kind is a decision for its PR, with do, check and undo written down here.

### What an action owns

**Never undo what the app didn't do.** Before an action does anything, its check runs. Done already is **found**: it is reported as done, and the journal records it as not owned, so a removal leaves it. A package that was installed, a group membership that existed, lingering that was on, a key that had the value: the app never takes them away. An action that changes part of a thing owns only that part: an `apt` action owns the packages it installed, not those it found; a `setting` owns the key, with its old value, never the file.

Each action records **claims**, one per thing it changed or found: `path:/…`, `dir:/…` (a folder it made for a path), `key:<file>#<key>`, `package:<name>`, `dep:<name>` (a package apt pulled in for the step's, always owned, kept by the step while it stands), `nix:<name>`, `member:<group>:<user>`, `group:<name>`, `unit:<scope>:<name>`, `linger:<user>`, `run`, each owned or not. A claim another app's journal also holds is handed over when its owner is removed, not undone, with the owner's prior state: an `apt` package two apps need is removed with the second, not the first (a `dep:` passes to an app whose step found that `package:`), and a unit one app enabled and another found enabled goes back to disabled when the second of them goes, whichever order they go in.

**Who owns a path comes from the journals.** An action that would write a path another app's journal owns (the path itself, a folder above it or a path in it) is refused before anything runs, naming the owner ("~/.config/nvim belongs to AstroNvim"). A path that is there and that no journal owns, and whose check isn't done, is the user's own: the action refuses and names it, and never overwrites it. A path already as the action would leave it is found, not owned. There is no hand-written `conflicts` field. A `setting`'s key follows the same rule against other apps; changing the user's own key is what a `setting` is for, and its undo puts their value back.

### The journal

Each app has one journal a side: `~/.local/state/kubaos/apps/<id>/journal.toml` for its user actions, `/var/lib/kubaos/apps/<id>/journal.toml` (root's, readable by everyone, written by root only) for its root actions. It is state, never in the picks: it describes this machine.

```toml
app = "t3-code"
setup = "service"
# Every step of the install is done. The user journal says it for both.
complete = true

[[entry]]
step = 6
state = "done"
claims = [{ claim = "run", owned = true }]
action = { name = "Start the service", kind = "run", do = ["/home/kuba/.local/bin/t3", "service", "install"], … }
```

An entry holds the action as it was done, its words filled in, so a removal undoes what was done even after a release changed the folder, and its **prior state**: what the undo needs and nothing else (a key's old value, whether the unit was enabled, whether lingering was on).

**Write-ahead.** Before an action runs, its entry is written as `doing`, with its claims and prior state read from the machine; once it is done, `done`. Before an undo, it is written `undoing`, and it goes once the undo is done; so a run killed during an undo leaves an entry that is never taken as settled. The file is rewritten whole and renamed into place, so a run killed at any point leaves a journal that is either the old one or the new one.

**Failure.** The steps already done are undone, newest first, across both journals; the machine is as before and the error names the step ("Start the service: `t3 service install` exited with code 1"). For an install, "before" is before the install began: an app that wasn't complete (a fresh install, or one resumed after a stop) is rolled back whole and its journals go. An app that was complete undoes only the steps this run added (a release added one), and stays installed: a step this run did again or updated stays done, and the next run checks it.

**Remove.** The journal is undone, newest first, and its folders go. Data the user made (Docker's images, T3's projects and threads) is no action's, and stays. An undo that fails stops the removal there; the journal keeps what is left, and the next removal goes on from it.

**Every run checks every step.** The journal says what the app owns, never whether a step still holds: a run checks each step, whatever its entry. A step whose check says done is reported done (settled, or found when the journal has no entry for it); one whose check fails is done again, as `kubaos update` repairs an app the user broke, keeping what its entry owned and the prior state recorded before it first ran. A step whose entry is `doing` or `undoing` (a run stopped during it) is checked the same way. So a run stopped after step 2 of 4 reports 1–2 done and goes on from 3, and a step after one that changed is checked like any other (Docker's `docker.service` `enable = false` after a new Docker enabled it again).

**A changed step.** A step is changed when its entry's action, as recorded, isn't the folder's action at that step as resolved now, word for word. A changed step that acts on the same thing (the same kind on the same side and, for a kind whose thing has a name, the same name: a `deb`'s package, a `file`'s, `symlink`'s or `output`'s path, a `setting`'s file and key, a `unit`'s unit and scope, a `group`'s group, a `web_app`'s id; any `apt`, `nix`, `download`, `linger` or `run` is the same thing as another of its kind) is **updated in place**: the new action is done over the old one when its check fails (a `deb` or `apt` installs the new version over the old, a `file` is rewritten, a `download` of a new version is fetched to its new path, or over the old one at a `to` without `@VERSION@`), keeping ownership of everything both claim (and of the folders the old one made that the new one's path is in); then what only the old one owned is undone (a package dropped from an `apt` list, the old version's download folder). The entry keeps the old action with those claims as `replaced` until that undo is done, so a run stopped between them finishes it. A changed step that acts on another thing is undone first, then the new action is done. An app is never undone and done again whole.

**A declined password changes nothing**: the root half is started before the first step whenever the run may need it, so a run that needs root and doesn't get it stops before anything runs. A run needs root when a root step has no entry in root's journal that is settled and records the same action word for word, when such a step's check, run from the user's side, doesn't say done (a root `run`'s check runs only as root, so such a step always needs it), and when an app that isn't complete has root steps a failure would undo. An installed app whose root steps all check done installs without a prompt.

**Installed** is the user journal's `complete`. Reading it is two small files an app, no dpkg query and no check run, so the Command menu's status reads stay cheap.

**Querying ownership** reads every journal (one TOML file an app and side, a few hundred bytes each) into an index of claims; a dozen apps is a dozen small reads, so no second index is kept that could drift from the journals.

### Events

Each step is an event of the run's `UpdateEvent`s: `action` with the app, the step, of how many, its name and its state (`done`, `found`, `skipped`, `undoing`, `undone`, `failed`). A `step` event carries the name and a `progress` event the count, so the Command menu's Job shows "Start the service" and how far along it is without knowing the new event.

### Root

**The root half is the `kubaos-apps` binary, not shell.** A root run starts one root process (`pkexec /usr/local/libexec/kubaos-apps serve` for `kubaos app`, through sudo inside `kubaos update`), one polkit prompt a run, which answers requests on its stdin, one JSON line each: `preflight` (an install's root steps checked against the root journals' claims), `step` (do one step of an app's setup) and `undo` (undo one step of the root journal). A request names an app id, a setup id and a step number, nothing else: the binary reads the action from the root-owned catalog (`/usr/share/kubaos/apps/<id>/app.toml`, refused unless the checkout, its `apps/` and `system/`, `system/apt-repo.sh`, and the app's folder and everything in it, at any depth, are root's, none a link, and no one else may write them), refuses a step whose action isn't root's (a user action, or a `run` without `root = true`), and undoes only what the root journal, which only root writes, recorded. It holds `/run/kubaos/apps.lock`, in a folder only root may write, while it serves, so only one run changes a machine at a time; the user's side holds `~/.local/state/kubaos/apps/.lock`.

Why a binary: the actions are Rust, typed and tested once; a shell root helper would have to implement the vocabulary a second time in the language least able to check it, which is how `kubaos-apps` grew to two hundred lines of manifest parsing in awk. The binary is small to audit because its surface is: its `main` takes `serve` and nothing else, and every request is three validated words; the actions it runs are the same code the tests prove. It is built with the release (`.#kubaos`, a second binary of `kubaos-cli`), linked from the Nix store into `/usr/local/libexec/` by root, as `install_scx` links `scx_lavd`, so neither the user nor their profile can change what pkexec runs.

### The six apps

| App | Actions |
|---|---|
| Claude Code | `apt` claude-code from repo `claude-code` |
| Google Chrome | `apt` google-chrome-stable from repo `google-chrome` (amd64); `file` `/usr/local/libexec/dot-chrome-theme` (`mode = "755"`); `file` `/etc/systemd/system/dot-chrome-theme.path` and `.service` from their templates (`@USER@`, `@HOME@`); `unit` system `dot-chrome-theme.path` enable + start; `unit` system `dot-chrome-theme.service` start; `output` `/etc/opt/chrome/policies/managed/dot-theme.json` |
| Docker | `apt` Docker's five packages from repo `docker`; `group` docker; `file` `/etc/systemd/system/docker.service.d/kubaos-stop.conf`; `unit` system `docker.service` and `containerd.service` `enable = false`; `unit` system `docker.socket` enable + start |
| Bitwarden | `deb` bitwarden at its pin, `self_updates`; `file` `/etc/xdg/autostart/bitwarden.desktop` |
| T3 Code, `desktop` | `deb` t3code at its pin, `self_updates` |
| T3 Code, `service` | the shared `deb`; `[[setup.service.ready]]` T3 Code is closed; `download` the t3 runtime to `~/.t3/runtime/versions/@VERSION@` (`tar-gz`, `strip = 1`); `file` `.install-complete` in it holding the version; `run` `t3 install` (check: `~/.local/bin/t3` is T3's; undo: the launchers go); `linger`; `run` `t3 service install` (check: `systemctl --user is-enabled t3code.service`; undo: `t3 service uninstall`); `setting` `~/.t3/userdata/desktop-settings.json` `localEnvironmentEnabled = false`; `run` pairing (`./pair`, a one-time link opened in T3 Code) with `window = "Open T3 Code and run t3 pair to connect it to the service on this machine"`, its check and undo T3's own pairing list and revoke (`t3 auth pairing`), confirmed against T3 when the app moves |

What changes in behaviour: cleanup is complete, every time. Removing an app undoes everything its journal owns, even what something outside the catalog would have used: Chrome's theme helper, when Chrome's install made it, goes with Chrome even while a Chrome beta the user installed reads it too; a helper found there (left by an older install, which the base-apps migration kept) was never Chrome's, and stays; Docker's group membership and drop-in go with Docker unless another app's journal claims them; T3 Code's lingering is restored by the `linger` action's prior state instead of a mark file, and only if it was off. The vendor's `google-chrome.list` is turned off by the `apt` action's repository setup, as ADR 0008's one-way trust already does.

## Alternatives

- **A scripting language for installs** (shell functions as today, or a Lua or Rhai script an app). Every script needs its own cleanup, written by hand and tested by nobody, and what it changed is unknowable to anything but itself: no rollback, no ownership, no resume. A sandboxed language could limit what a script touches but still couldn't say how to undo it. The vocabulary trades expressiveness for knowing; `run` covers the rest, with its inverse declared.
- **Nix for everything.** Nix owns the home and a switch is atomic, but the six apps are mostly outside it: apt packages from vendor repositories, a vendor's `.deb` that updates itself, a polkit-era system unit, group membership, an Electron app's own settings file. A NixOS module would do it on NixOS; on Ubuntu, home-manager can't add a user to a group or install a `.deb`. `nix` stays one kind among the others.
- **btrfs snapshots** as the rollback. A snapshot of `@` rolls back everything on the machine, the user's own changes made meanwhile included, needs a reboot to restore, and says nothing about what an app owns, so a removal weeks later can't use it. KubaOS keeps its snapper snapshots for updates (ADR 0001); an app's box is its journal.
- **Containers or Flatpak.** A sandboxed app owns its files by construction, but the six apps are those that need the machine: Docker's daemon, Bitwarden's SSH agent and system-authentication unlock, Chrome's policies, Claude Code and T3 Code running the user's tools. ADR 0008 already excludes Flatpak.

## Consequences

- The engine, its journal and the `kubaos-apps` binary land first, proven on fixture apps. The six apps move to `apps/<id>/` in the next piece of #435, with the binary's install as root (`install_apps_helper`, a migration) and `kubaos app` and `kubaos update` running the engine; `system_hooks`, `user_hooks`, `apps/user-hooks.sh`, `apps/user-helper`, the shell `kubaos-apps` and the apps' functions in `system/lib.sh` go then, and a migration adopts what each app's checks find already on an installed machine as owned, without running anything.
- Dependencies (`requires`) and the configure stage come after that, in the format reserved above.
- Suites (#32) may share the vocabulary later: a Suite asked for a small, auditable set of privileged steps, which this is.
- The root journal is one a machine, not one a user: KubaOS is a one-person machine, and the root half acts for the user who asked.
