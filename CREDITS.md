# Credits

KubaOS credits every external tool, library, asset, font, data source and service it uses: the people who made them deserve it. This file is where those credits live, each with its creators, its licence and a link. Anything external a pull request adds gets its line here in the same pull request.

It credits what KubaOS uses directly: the crates its `Cargo.toml` files name, the npm packages `design/mockups/package.json` names, and the programs, fonts, assets and services the system and its development name. What those pull in, and the exact version of each, is in `Cargo.lock`, `flake.lock` and `design/mockups/package-lock.json`. `just credits-check` fails when a crate or npm package the manifests name has no line here.

## What KubaOS is built on

What KubaOS ships, draws on or is designed around.

### Base system

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| Btrfs tools (`btrfs-progs`) | The subvolumes the snapshots and the install are laid out in | Oracle, Red Hat, STRATO AG and the btrfs-progs contributors | GPL-2.0 | [git.kernel.org/…/kdave/btrfs-progs.git](https://git.kernel.org/pub/scm/linux/kernel/git/kdave/btrfs-progs.git) |
| GRUB | The boot menu, themed by KubaOS (`system/files/grub/`) | Free Software Foundation | GPL-3.0-or-later | [gnu.org/software/grub](https://www.gnu.org/software/grub/) |
| grub-btrfs | The snapshots in the boot menu | Antynea | GPL-3.0 | [github.com/Antynea/grub-btrfs](https://github.com/Antynea/grub-btrfs) |
| Home Manager | Builds the user's home from `home/`: programs, settings and services | Home Manager contributors | MIT | [github.com/nix-community/home-manager](https://github.com/nix-community/home-manager) |
| inotify-tools | Lets grub-btrfs see new snapshots | John McCutchan, Rohan McGovern, Radu Voicilas and Enrico Maria Crisostomo | GPL-2.0 | [github.com/rvoicilas/inotify-tools](https://github.com/rvoicilas/inotify-tools) |
| keyd | The accent key: holds a letter to type its accented forms | Raheman Vaiya | MIT | [github.com/rvaiya/keyd](https://github.com/rvaiya/keyd) |
| Nix | Installs the home's packages, pinned by `flake.lock` | NixOS | LGPL-2.1-or-later | [github.com/NixOS/nix](https://github.com/NixOS/nix) |
| nixpkgs | Every package the home installs or KubaOS curates (`home/curated.nix`) | Eelco Dolstra and the Nixpkgs/NixOS contributors | MIT | [github.com/NixOS/nixpkgs](https://github.com/NixOS/nixpkgs) |
| Plymouth | The boot splash, where the neon sign switches on | Kristian Høgsberg, Ray Strode, Charlie Brej and Red Hat | GPL-2.0-or-later | [freedesktop.org/wiki/Software/Plymouth](https://www.freedesktop.org/wiki/Software/Plymouth/) |
| `scx_lavd` (sched_ext schedulers) | The latency-aware CPU scheduler KubaOS runs | sched-ext | GPL-2.0-only | [github.com/sched-ext/scx](https://github.com/sched-ext/scx) |
| Snapper | The system's snapshots before every update | Novell, Inc. | GPL-2.0-only | [snapper.io](http://snapper.io) |
| TUXEDO drivers | Keyboard, fan and power support on TUXEDO laptops | TUXEDO Computers GmbH | GPL-2.0-or-later | [gitlab.com/tuxedocomputers/…/tuxedo-drivers](https://gitlab.com/tuxedocomputers/development/packages/tuxedo-drivers) |
| Ubuntu | The base system KubaOS installs on and updates with apt | Canonical Ltd. | Each package's own | [ubuntu.com](https://www.ubuntu.com/) |

### Desktop

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| adw-gtk3 | GTK 3 apps' theme, recoloured from the wallpaper | lassekongo83 | LGPL-2.1-only | [github.com/lassekongo83/adw-gtk3](https://github.com/lassekongo83/adw-gtk3) |
| Caelestia | The structure the shell follows: one frame with panels growing out of its edges | caelestia-dots | GPL-3.0 | [github.com/caelestia-dots/shell](https://github.com/caelestia-dots/shell) |
| foot | The fallback terminal, for when Ghostty can't start | Daniel Eklöf | MIT | [codeberg.org/dnkl/foot](https://codeberg.org/dnkl/foot/) |
| fuzzel | The fallback launcher | Daniel Eklöf | MIT, Zlib | [codeberg.org/dnkl/fuzzel](https://codeberg.org/dnkl/fuzzel) |
| GNOME Keyring | Holds the VPN and app secrets (`secret-tool`) | Collabora Ltd., Nokia Corporation and the GNOME Keyring contributors | LGPL-2.1-or-later, GPL-2.0-or-later | [gitlab.gnome.org/GNOME/gnome-keyring](https://gitlab.gnome.org/GNOME/gnome-keyring) |
| GPU Screen Recorder | Screen recording | dec05eba and contributors | GPL-3.0-only | [git.dec05eba.com/gpu-screen-recorder](https://git.dec05eba.com/gpu-screen-recorder/about/) |
| grim | Screenshots | Simon Ser | MIT | [gitlab.freedesktop.org/emersion/grim](https://gitlab.freedesktop.org/emersion/grim) |
| hyprcursor | The cursor format `kubaos cursor` writes its recoloured pointer in | Vaxry (vaxerski) and the Hyprland contributors | BSD-3-Clause | [github.com/hyprwm/hyprcursor](https://github.com/hyprwm/hyprcursor) |
| hypridle | Locks, turns the screens off and suspends on idle, as `kubaos-idle` sets it | Vaxry (vaxerski) and the Hyprland contributors | BSD-3-Clause | [github.com/hyprwm/hypridle](https://github.com/hyprwm/hypridle) |
| Hyprland | The compositor: windows, workspaces, keys and animation (`config/hypr/`) | Vaxry (vaxerski) and the Hyprland contributors | BSD-3-Clause | [github.com/hyprwm/Hyprland](https://github.com/hyprwm/Hyprland) |
| hyprland-guiutils | Hyprland's own dialogs | Vaxry (vaxerski) and the Hyprland contributors | BSD-3-Clause | [github.com/hyprwm/hyprland-guiutils](https://github.com/hyprwm/hyprland-guiutils) |
| hyprlock | The lock screen's fallback | Vaxry (vaxerski) and the Hyprland contributors | BSD-3-Clause | [github.com/hyprwm/hyprlock](https://github.com/hyprwm/hyprlock) |
| hyprpicker | The colour picker | Vaxry (vaxerski) and the Hyprland contributors | BSD-3-Clause | [github.com/hyprwm/hyprpicker](https://github.com/hyprwm/hyprpicker) |
| hyprpolkitagent | The polkit agent behind the shell's password prompt | Vaxry (vaxerski) and the Hyprland contributors | BSD-3-Clause | [github.com/hyprwm/hyprpolkitagent](https://github.com/hyprwm/hyprpolkitagent) |
| libnotify (`notify-send`) | Notifications sent from the scripts | Christian Hammond and John Palmieri | LGPL-2.1 | [gitlab.gnome.org/GNOME/libnotify](https://gitlab.gnome.org/GNOME/libnotify) |
| matugen | The colour scheme drawn from the wallpaper, for the shell and every themed app | InioX | GPL-2.0-or-later | [github.com/InioX/matugen](https://github.com/InioX/matugen) |
| playerctl | Media keys and the shell's player controls | Tony Crisci | LGPL-3.0 | [github.com/acrisci/playerctl](https://github.com/acrisci/playerctl) |
| polkit | Lets the shell and Kōbō ask for an administrator's password | David Zeuthen | LGPL-2.0-or-later | [github.com/polkit-org/polkit](https://github.com/polkit-org/polkit) |
| Qalculate! (`qalc`) | The fallback calculator | Hanna Knutsson | GPL-2.0-or-later | [qalculate.github.io](http://qalculate.github.io) |
| Qt 6 | The toolkit the shell and `KubaOS.Components` run on, with Qt Multimedia, Qt 5 Compat and Qt Image Formats | The Qt Company Ltd. | LGPL-3.0-or-later, GPL-2.0-or-later | [qt.io](https://www.qt.io/) |
| qt5ct | Qt 5 apps' theme and font settings | trialuser02 | BSD | [sourceforge.net/projects/qt5ct](https://sourceforge.net/projects/qt5ct/) |
| qt6ct | Qt 6 apps' theme and font settings | trialuser | BSD-2-Clause | [opencode.net/trialuser/qt6ct](https://www.opencode.net/trialuser/qt6ct) |
| Quickshell | The desktop shell's runtime: the bar, panels, lock screen and Command menu are QML on it (`config/quickshell/`) | outfoxxed | LGPL-3.0 | [quickshell.outfoxxed.me](https://quickshell.outfoxxed.me) |
| Satty | Annotates screenshots | Matthias Gabriel | MPL-2.0 | [github.com/gabm/Satty](https://github.com/gabm/Satty) |
| slurp | Picks a region for screenshots and recordings | Simon Ser and Drew DeVault | MIT | [github.com/emersion/slurp](https://github.com/emersion/slurp) |
| socat | Talks to Hyprland's socket from the scripts | Gerhard Rieger and contributors | GPL-2.0-only | [dest-unreach.org/socat](http://www.dest-unreach.org/socat/) |
| wl-clipboard | Copies from the scripts (`wl-copy`) | Sergey Bugaev | GPL-3.0-or-later | [github.com/bugaevc/wl-clipboard](https://github.com/bugaevc/wl-clipboard) |
| Zenity | Questions from apps that can't draw a window of their own | Glynn Foster and contributors | LGPL-2.1-or-later | [gitlab.gnome.org/GNOME/zenity](https://gitlab.gnome.org/GNOME/zenity) |

### Devices, sound and network

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| Bitwarden CLI (`bw`) | The vault the secrets and the YubiKey setup read from | Bitwarden | See its repository | [github.com/bitwarden/clients](https://github.com/bitwarden/clients) |
| BlueZ | Bluetooth, which `kubaos-bluetooth` pairs through | Intel Corporation and others | GPL-2.0-or-later | [bluez.org](https://www.bluez.org/) |
| brightnessctl | Screen and keyboard brightness | Mykyta Holuakha | MIT | [github.com/Hummer12007/brightnessctl](https://github.com/Hummer12007/brightnessctl) |
| ddcutil | External monitors' brightness | Sanford Rockowitz | GPL-2.0-or-later | [ddcutil.com](http://www.ddcutil.com/) |
| fprintd | Fingerprint sign-in | Bastien Nocera and Daniel Drake | GPL-2.0-or-later | [fprint.freedesktop.org](https://fprint.freedesktop.org/) |
| KDE Connect | The phone link (`kubaos-phone`) | KDE | GPL-2.0-or-later and others | [invent.kde.org/network/kdeconnect-kde](https://invent.kde.org/network/kdeconnect-kde) |
| libfido2 (`fido2-tools`) | Sets up the security key's PIN and credentials | Yubico AB | BSD-2-Clause | [github.com/yubico/libfido2](https://github.com/yubico/libfido2) |
| NetworkManager | Wi-Fi, wired, VPN and the proxy (`kubaos-proxy`) | Red Hat, Inc. and Novell, Inc. | GPL-2.0-or-later | [networkmanager.dev](https://networkmanager.dev) |
| noise-suppression-for-voice | The microphone's noise suppression (`home/files/noise-suppression.conf`) | werman | GPL-3.0 | [github.com/werman/noise-suppression-for-voice](https://github.com/werman/noise-suppression-for-voice) |
| pam-u2f | Signs in and unlocks `sudo` with the security key | Yubico AB | BSD-2-Clause | [developers.yubico.com/pam-u2f](https://developers.yubico.com/pam-u2f/) |
| PipeWire | Sound and screen sharing | Wim Taymans and the PipeWire Developers | MIT | [pipewire.org](https://pipewire.org/) |
| power-profiles-daemon | The power modes `kubaos-power` switches | Bastien Nocera, Marco Trevisan and Mario Limonciello | GPL-3.0-or-later | [gitlab.freedesktop.org/upower/power-profiles-daemon](https://gitlab.freedesktop.org/upower/power-profiles-daemon) |
| UDisks | Mounts removable drives (`kubaos-automount`) | David Zeuthen, Martin Pitt and Tomas Bzatek | LGPL-2.0-or-later, GPL-2.0-or-later | [freedesktop.org/wiki/Software/udisks](https://www.freedesktop.org/wiki/Software/udisks/) |
| UPower | Battery state for the bar and the power stops | David Zeuthen and Richard Hughes | GPL-2.0-or-later | [upower.freedesktop.org](https://upower.freedesktop.org/) |
| WirePlumber | PipeWire's session manager, configured in `config/wireplumber/` | Collabora Ltd. | MIT | [pipewire.org](https://pipewire.org) |
| YubiKey Manager (`ykman`) | Reads and sets up the YubiKey | Dain Nilsson (Yubico) | BSD-2-Clause | [developers.yubico.com/yubikey-manager](https://developers.yubico.com/yubikey-manager) |

### Files and media

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| 7-Zip | Opens and makes archives | Igor Pavlov | LGPL-2.1-or-later | [7-zip.org](https://www.7-zip.org) |
| FFmpeg | Video wallpapers' stills and media conversion | Fabrice Bellard and the FFmpeg project | LGPL-2.1-or-later, GPL-2.0-or-later | [ffmpeg.org](https://www.ffmpeg.org/) |
| ffmpegthumbnailer | Video thumbnails in Files | Dirk Vanden Boer | GPL-2.0-or-later | [github.com/dirkvdb/ffmpegthumbnailer](https://github.com/dirkvdb/ffmpegthumbnailer) |
| File Roller | Archives from Files | Free Software Foundation, Inc. | GPL-2.0-or-later | [gitlab.gnome.org/GNOME/file-roller](https://gitlab.gnome.org/GNOME/file-roller) |
| glycin | Image thumbnails in Files | Sophie Herold | LGPL-2.1-or-later OR MPL-2.0 | [gitlab.gnome.org/GNOME/glycin](https://gitlab.gnome.org/GNOME/glycin) |
| GNOME Font Viewer | Previews fonts | James Henstridge and Cosimo Cecchi | GPL-2.0-or-later | [download.gnome.org/sources/gnome-font-viewer](https://download.gnome.org/sources/gnome-font-viewer/) |
| GStreamer plugins | Media playback in the GNOME apps and the shell | Contributors to the GStreamer project | LGPL-2.0-or-later | [gstreamer.freedesktop.org](http://gstreamer.freedesktop.org) |
| GVfs | Network places and trash in Files | Red Hat, Inc., CodeFactory AB and the GVfs contributors | LGPL-2.0-or-later | [download.gnome.org/sources/gvfs](https://download.gnome.org/sources/gvfs) |
| ImageMagick | Wallpaper thumbnails and blurs, and Kōbō's pictures (`make-assets.sh`) | ImageMagick Studio LLC and John Cristy | Apache-2.0 | [imagemagick.org](http://www.imagemagick.org/) |
| Loupe | The image viewer | The Loupe Team | GPL-3.0-or-later | [gitlab.gnome.org/GNOME/loupe](https://gitlab.gnome.org/GNOME/loupe) |
| Nautilus | Files, with KubaOS's extensions (`home/files/kubaos-nautilus.py`) | Eazel, Inc., Red Hat, Inc. and the Nautilus contributors | GPL-3.0-or-later | [apps.gnome.org/Nautilus](https://apps.gnome.org/Nautilus/) |
| nautilus-python | Runs KubaOS's Files extensions | Johan Dahlin and Novell, Inc. | GPL-2.0-or-later | [download.gnome.org/sources/nautilus-python](https://download.gnome.org/sources/nautilus-python/) |
| Papers | The document viewer | Marco Pesenti Gritti, Christian Persch, Carlos Garcia Campos and Red Hat | GPL-2.0-or-later | [gitlab.gnome.org/GNOME/papers](https://gitlab.gnome.org/GNOME/papers) |
| ripdrag | Drags files out of yazi | Nicola Guerrera | GPL-3.0-only | [github.com/nik012003/ripdrag](https://github.com/nik012003/ripdrag) |
| Showtime | The video player | Laura Kramolis | GPL-3.0-or-later | [apps.gnome.org/Showtime](https://apps.gnome.org/Showtime) |
| Sushi | Quick previews in Files | Red Hat, Inc. | GPL-2.0-or-later | [gitlab.gnome.org/GNOME/sushi](https://gitlab.gnome.org/GNOME/sushi) |
| The Unarchiver (`unar`) | Opens RAR and other archives | MacPaw Inc. | LGPL-2.1-or-later | [github.com/MacPaw/XADMaster](https://github.com/MacPaw/XADMaster) |
| yazi | The terminal file manager, launched by `kubaos-files` | sxyazi | MIT | [github.com/sxyazi/yazi](https://github.com/sxyazi/yazi) |

### Terminal and shell

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| Atuin | The shell's history search | Ellie Huxtable, Atuin, Inc. | MIT | [github.com/atuinsh/atuin](https://github.com/atuinsh/atuin) |
| bat | `cat` with highlighting, and fzf's file previews | David Peter (sharkdp) | Apache-2.0, MIT | [github.com/sharkdp/bat](https://github.com/sharkdp/bat) |
| btop | The process monitor (`config/btop/`) | aristocratos | Apache-2.0 | [github.com/aristocratos/btop](https://github.com/aristocratos/btop) |
| delta | Git's diffs, themed from the wallpaper | Dan Davison | MIT | [github.com/dandavison/delta](https://github.com/dandavison/delta) |
| dust | Shows what fills a disk | bootandy | Apache-2.0 | [github.com/bootandy/dust](https://github.com/bootandy/dust) |
| dysk | Lists disks and their space | dystroy (Denys Séguret) | MIT | [github.com/Canop/dysk](https://github.com/Canop/dysk) |
| eza | `ls`, with icons and the wallpaper's colours | Christina Sørensen and the eza contributors | EUPL-1.2 | [github.com/eza-community/eza](https://github.com/eza-community/eza) |
| fastfetch | The terminal's greeting (`config/fastfetch/`) | Linus Dierheimer and Carter Li | MIT | [github.com/fastfetch-cli/fastfetch](https://github.com/fastfetch-cli/fastfetch) |
| fd | `find`, for the shell and fzf | David Peter (sharkdp) | Apache-2.0, MIT | [github.com/sharkdp/fd](https://github.com/sharkdp/fd) |
| fzf | Fuzzy finding in the shell | Junegunn Choi | MIT | [github.com/junegunn/fzf](https://github.com/junegunn/fzf) |
| Ghostty | The terminal (`config/ghostty/`) | Mitchell Hashimoto and the Ghostty contributors | MIT | [ghostty.org](https://ghostty.org/) |
| Git | Version control, and the updates `kubaos-git` runs | Linus Torvalds and others | GPL-2.0 | [git-scm.com](https://git-scm.com/) |
| GitHub CLI (`gh`) | GitHub from the terminal | GitHub, Inc. | MIT | [github.com/cli/cli](https://github.com/cli/cli) |
| glow | Renders markdown in the terminal | Charmbracelet, Inc. | MIT | [github.com/charmbracelet/glow](https://github.com/charmbracelet/glow) |
| jq | JSON in the scripts and the shell | Stephen Dolan | MIT | [jqlang.github.io/jq](https://jqlang.github.io/jq/) |
| lazydocker | Docker in the terminal | Jesse Duffield | MIT | [github.com/jesseduffield/lazydocker](https://github.com/jesseduffield/lazydocker) |
| lazygit | Git in the terminal (`config/lazygit/`) | Jesse Duffield | MIT | [github.com/jesseduffield/lazygit](https://github.com/jesseduffield/lazygit) |
| mise | The language toolchains of the Suites | Jeff Dickey | MIT | [mise.jdx.dev](https://mise.jdx.dev) |
| Oh My Posh | The prompt (`config/ohmyposh/`) | Jan De Dobbeleer | MIT | [ohmyposh.dev](https://ohmyposh.dev) |
| Oh My Zsh | Zsh's git plugin and completions | Planet Argon and the Oh My Zsh contributors | MIT | [ohmyz.sh](https://ohmyz.sh/) |
| ripgrep | `grep`, for the shell and Neovim | Andrew Gallant | Unlicense, MIT | [github.com/BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) |
| tealdeer | `tldr` pages | Danilo Bargen and contributors | Apache-2.0, MIT | [github.com/tealdeer-rs/tealdeer](https://github.com/tealdeer-rs/tealdeer) |
| tmux | The terminal multiplexer (`config/tmux/`) | Nicholas Marriott | BSD-3-Clause | [tmux.github.io](https://tmux.github.io/) |
| tmux-better-mouse-mode | Scrolling in tmux with the mouse | Nathan Daly | MIT | [github.com/NHDaly/tmux-better-mouse-mode](https://github.com/NHDaly/tmux-better-mouse-mode) |
| tmux-sensible | tmux's base settings | Bruno Sutic | MIT | [github.com/tmux-plugins/tmux-sensible](https://github.com/tmux-plugins/tmux-sensible) |
| tmux-sessionizer (`tms`) | Opens projects as tmux sessions (`config/tms/`) | Jared Moulton | MIT | [github.com/jrmoulton/tmux-sessionizer](https://github.com/jrmoulton/tmux-sessionizer) |
| vim-tmux-navigator | Moves between Neovim splits and tmux panes with the same keys | Chris Toomey | MIT | [github.com/christoomey/vim-tmux-navigator](https://github.com/christoomey/vim-tmux-navigator) |
| xh | HTTP requests from the terminal | Mohamed Daahir | MIT | [github.com/ducaale/xh](https://github.com/ducaale/xh) |
| yq | YAML in the scripts and the shell | Mike Farah | MIT | [mikefarah.gitbook.io/yq](https://mikefarah.gitbook.io/yq/) |
| zoxide | `cd` that learns the folders used most | Ajeet D'Souza | MIT | [github.com/ajeetdsouza/zoxide](https://github.com/ajeetdsouza/zoxide) |
| Zsh | The shell | Paul Falstad and the Zsh Development Group | MIT-Modern-Variant | [zsh.org](https://www.zsh.org/) |

### Editor

KubaOS's Neovim (`config/nvim/`) is LazyVim with these plugins (`config/nvim/lazy-lock.json`) and language tools.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| aerial.nvim | The code outline | Steven Arcangeli | MIT | [github.com/stevearc/aerial.nvim](https://github.com/stevearc/aerial.nvim) |
| bash-language-server | Shell scripts' language server | bash-lsp | MIT | [github.com/bash-lsp/bash-language-server](https://github.com/bash-lsp/bash-language-server) |
| blink.cmp | Completion | Liam Dyer | MIT | [github.com/saghen/blink.cmp](https://github.com/saghen/blink.cmp) |
| bufferline.nvim | The tab line of open files | Akin Sowemimo | GPL-3.0 | [github.com/akinsho/bufferline.nvim](https://github.com/akinsho/bufferline.nvim) |
| catppuccin | A colour scheme LazyVim installs | Catppuccin | MIT | [github.com/catppuccin/nvim](https://github.com/catppuccin/nvim) |
| conform.nvim | Formatting on save | Steven Arcangeli | MIT | [github.com/stevearc/conform.nvim](https://github.com/stevearc/conform.nvim) |
| crates.nvim | Crate versions in `Cargo.toml` | Tobias Schmitz | MIT | [github.com/Saecki/crates.nvim](https://github.com/Saecki/crates.nvim) |
| dial.nvim | Increments dates, booleans and numbers | Mogami Shinichi | MIT | [github.com/monaqa/dial.nvim](https://github.com/monaqa/dial.nvim) |
| diffview.nvim | Git diffs and file history | Sindre T. Strøm | GPL-3.0-or-later | [github.com/sindrets/diffview.nvim](https://github.com/sindrets/diffview.nvim) |
| docker-compose-language-service | Compose files' language server | Microsoft Corporation | MIT | [github.com/microsoft/compose-language-service](https://github.com/microsoft/compose-language-service) |
| dockerfile-language-server | Dockerfiles' language server | Remy Suen | MIT | [github.com/rcjsuen/dockerfile-language-server](https://github.com/rcjsuen/dockerfile-language-server) |
| flash.nvim | Jumps to any place on screen | Folke Lemaitre | Apache-2.0 | [github.com/folke/flash.nvim](https://github.com/folke/flash.nvim) |
| friendly-snippets | Snippets | Rafael Madriz | MIT | [github.com/rafamadriz/friendly-snippets](https://github.com/rafamadriz/friendly-snippets) |
| gitsigns.nvim | Git changes in the gutter | Lewis Russell | MIT | [github.com/lewis6991/gitsigns.nvim](https://github.com/lewis6991/gitsigns.nvim) |
| grug-far.nvim | Search and replace across files | Stephan Badragan | MIT | [github.com/MagicDuck/grug-far.nvim](https://github.com/MagicDuck/grug-far.nvim) |
| hadolint | Lints Dockerfiles | Lukas Martinelli | GPL-3.0 | [github.com/hadolint/hadolint](https://github.com/hadolint/hadolint) |
| inc-rename.nvim | Renames with a live preview | Jonas Strittmatter | MIT | [github.com/smjonas/inc-rename.nvim](https://github.com/smjonas/inc-rename.nvim) |
| lazy.nvim | The plugin manager | Folke Lemaitre | Apache-2.0 | [github.com/folke/lazy.nvim](https://github.com/folke/lazy.nvim) |
| lazydev.nvim | Lua completion for Neovim's own API | Folke Lemaitre | Apache-2.0 | [github.com/folke/lazydev.nvim](https://github.com/folke/lazydev.nvim) |
| LazyVim | The Neovim distribution KubaOS's config builds on | Folke Lemaitre | Apache-2.0 | [github.com/LazyVim/LazyVim](https://github.com/LazyVim/LazyVim) |
| lua-language-server | Lua's language server | 最萌小汐 | MIT | [github.com/luals/lua-language-server](https://github.com/luals/lua-language-server) |
| lualine.nvim | The status line, themed from the wallpaper | hoob3rt | MIT | [github.com/nvim-lualine/lualine.nvim](https://github.com/nvim-lualine/lualine.nvim) |
| markdown-preview.nvim | Previews markdown in a browser | 年糕小豆汤 (iamcco) | MIT | [github.com/iamcco/markdown-preview.nvim](https://github.com/iamcco/markdown-preview.nvim) |
| markdown-toc | Writes markdown tables of contents | Jon Schlinkert | MIT | [github.com/jonschlinkert/markdown-toc](https://github.com/jonschlinkert/markdown-toc) |
| markdownlint-cli2 | Lints markdown | David Anson | MIT | [github.com/DavidAnson/markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) |
| marksman | Markdown's language server | Artem Pyanykh | MIT | [github.com/artempyanykh/marksman](https://github.com/artempyanykh/marksman) |
| mason-lspconfig.nvim | Connects Mason's servers to Neovim | William Boman | Apache-2.0 | [github.com/mason-org/mason-lspconfig.nvim](https://github.com/mason-org/mason-lspconfig.nvim) |
| mason-nvim-dap.nvim | Connects Mason's debug adapters to nvim-dap | Jay Patel | AGPL-3.0 | [github.com/jay-babu/mason-nvim-dap.nvim](https://github.com/jay-babu/mason-nvim-dap.nvim) |
| mason.nvim | Installs the language tools that aren't npm packages | William Boman | Apache-2.0 | [github.com/mason-org/mason.nvim](https://github.com/mason-org/mason.nvim) |
| mini.nvim modules (ai, base16, hipatterns, icons, move, pairs, surround) | Text objects, the wallpaper's base16 colours, colour highlights, icons, moving lines, pairs and surrounds | Evgeni Chasnovski | MIT | [github.com/nvim-mini](https://github.com/nvim-mini) |
| neotest | Runs tests from the editor | Rónán Carrigan | MIT | [github.com/nvim-neotest/neotest](https://github.com/nvim-neotest/neotest) |
| Neovim | The editor | Thiago de Arruda and the Neovim contributors, on Bram Moolenaar's Vim | Apache-2.0, Vim | [neovim.io](https://neovim.io) |
| noice.nvim | The command line and messages UI | Folke Lemaitre | Apache-2.0 | [github.com/folke/noice.nvim](https://github.com/folke/noice.nvim) |
| nui.nvim | UI parts other plugins draw with | Munif Tanjim | MIT | [github.com/MunifTanjim/nui.nvim](https://github.com/MunifTanjim/nui.nvim) |
| nvim-dap | Debugging | Mathias Fussenegger | GPL-3.0 | [github.com/mfussenegger/nvim-dap](https://github.com/mfussenegger/nvim-dap) |
| nvim-dap-ui | The debugger's panes | Rónán Carrigan | MIT | [github.com/rcarriga/nvim-dap-ui](https://github.com/rcarriga/nvim-dap-ui) |
| nvim-dap-virtual-text | Variables' values inline while debugging | Stephan Seitz | GPL-3.0 | [github.com/theHamsta/nvim-dap-virtual-text](https://github.com/theHamsta/nvim-dap-virtual-text) |
| nvim-lint | Runs linters | Mathias Fussenegger | GPL-3.0 | [github.com/mfussenegger/nvim-lint](https://github.com/mfussenegger/nvim-lint) |
| nvim-lspconfig | Language servers' settings | Ashkan Kiani, Michael Lingelbach, Justin M. Keyes and contributors | Apache-2.0 | [github.com/neovim/nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) |
| nvim-nio | The async library neotest runs on | Rónán Carrigan | MIT | [github.com/nvim-neotest/nvim-nio](https://github.com/nvim-neotest/nvim-nio) |
| nvim-treesitter | Syntax trees for highlighting and folding | Thomas Vigouroux, Stephan Seitz, Christian Clason and contributors | Apache-2.0 | [github.com/nvim-treesitter/nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter) |
| nvim-treesitter-textobjects | Text objects from syntax trees | Stephan Seitz | Apache-2.0 | [github.com/nvim-treesitter/nvim-treesitter-textobjects](https://github.com/nvim-treesitter/nvim-treesitter-textobjects) |
| nvim-ts-autotag | Closes and renames HTML tags | windwp | MIT | [github.com/windwp/nvim-ts-autotag](https://github.com/windwp/nvim-ts-autotag) |
| oil.nvim | Edits folders as buffers | Steven Arcangeli | MIT | [github.com/stevearc/oil.nvim](https://github.com/stevearc/oil.nvim) |
| persistence.nvim | Restores sessions | Folke Lemaitre | Apache-2.0 | [github.com/folke/persistence.nvim](https://github.com/folke/persistence.nvim) |
| plenary.nvim | The Lua library other plugins use | TJ DeVries | MIT | [github.com/nvim-lua/plenary.nvim](https://github.com/nvim-lua/plenary.nvim) |
| Prettier | Formats markdown, JSON, YAML and the web languages | James Long and contributors | MIT | [prettier.io](https://prettier.io/) |
| render-markdown.nvim | Renders markdown in the buffer | Vlad (MeanderingProgrammer) | MIT | [github.com/MeanderingProgrammer/render-markdown.nvim](https://github.com/MeanderingProgrammer/render-markdown.nvim) |
| rustaceanvim | Rust in the editor, for the Rust Suite | Marc Jakobi | GPL-2.0 | [github.com/mrcjkb/rustaceanvim](https://github.com/mrcjkb/rustaceanvim) |
| SchemaStore.nvim | JSON and YAML schemas | b0o | Apache-2.0 | [github.com/b0o/SchemaStore.nvim](https://github.com/b0o/SchemaStore.nvim) |
| shfmt | Formats shell scripts | Daniel Martí | BSD-3-Clause | [github.com/mvdan/sh](https://github.com/mvdan/sh) |
| snacks.nvim | The explorer, the picker and the dashboard | Folke Lemaitre | Apache-2.0 | [github.com/folke/snacks.nvim](https://github.com/folke/snacks.nvim) |
| stylua | Formats Lua | johnnymorganz | MPL-2.0 | [github.com/johnnymorganz/stylua](https://github.com/johnnymorganz/stylua) |
| taplo | TOML's language server and formatter | Ferenc Tamás | MIT | [github.com/tamasfe/taplo](https://github.com/tamasfe/taplo) |
| todo-comments.nvim | Highlights and lists TODO comments | Folke Lemaitre | Apache-2.0 | [github.com/folke/todo-comments.nvim](https://github.com/folke/todo-comments.nvim) |
| tokyonight.nvim | A colour scheme LazyVim installs | Folke Lemaitre | Apache-2.0 | [github.com/folke/tokyonight.nvim](https://github.com/folke/tokyonight.nvim) |
| tree-sitter (CLI) | Builds the parsers nvim-treesitter installs | tree-sitter | MIT | [github.com/tree-sitter/tree-sitter](https://github.com/tree-sitter/tree-sitter) |
| trouble.nvim | Lists diagnostics and references | Folke Lemaitre | Apache-2.0 | [github.com/folke/trouble.nvim](https://github.com/folke/trouble.nvim) |
| ts-comments.nvim | Comments by the syntax tree's language | Folke Lemaitre | Apache-2.0 | [github.com/folke/ts-comments.nvim](https://github.com/folke/ts-comments.nvim) |
| vim-fugitive | Git commands in the editor | Tim Pope | Vim | [github.com/tpope/vim-fugitive](https://github.com/tpope/vim-fugitive) |
| vim-visual-multi | Multiple cursors | Gianmaria Bajo | MIT | [github.com/mg979/vim-visual-multi](https://github.com/mg979/vim-visual-multi) |
| vscode-langservers-extracted | JSON's language server, and eslint's for the Node Suite | hrsh7th | MIT | [github.com/hrsh7th/vscode-langservers-extracted](https://github.com/hrsh7th/vscode-langservers-extracted) |
| which-key.nvim | Shows the keys that follow a prefix | Folke Lemaitre | Apache-2.0 | [github.com/folke/which-key.nvim](https://github.com/folke/which-key.nvim) |
| yaml-language-server | YAML's language server | Red Hat Inc. and others | MIT | [github.com/redhat-developer/yaml-language-server](https://github.com/redhat-developer/yaml-language-server) |
| yanky.nvim | The yank history | Gilles Roustan | WTFPL | [github.com/gbprod/yanky.nvim](https://github.com/gbprod/yanky.nvim) |

### Suites

What the developer Suites (`home/suites/`) add beyond the toolchains mise installs.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| bacon | The Rust Suite's background checker (`config/bacon/`) | Canop | AGPL-3.0-only | [github.com/Canop/bacon](https://github.com/Canop/bacon) |
| BiRefNet | Cuts a wallpaper's foreground out, for the depth Suite | Peng Zheng and co-authors | MIT | [github.com/ZhengPeng7/BiRefNet](https://github.com/ZhengPeng7/BiRefNet) |
| Depth Anything V2 Small | The wallpapers' depth maps, in the depth Suite and the Lock mockup | Lihe Yang, Bingyi Kang, Zilong Huang and co-authors | Apache-2.0 | [github.com/DepthAnything/Depth-Anything-V2](https://github.com/DepthAnything/Depth-Anything-V2) |
| ONNX Runtime | Runs the wallpaper depth models (`kubaos-depth`, through `ort`) | microsoft | MIT | [github.com/microsoft/onnxruntime](https://github.com/microsoft/onnxruntime) |
| PHP Debug (`vscode-php-debug`) | The PHP Suite's Xdebug adapter | Felix Frederick Becker | MIT | [github.com/xdebug/vscode-php-debug](https://github.com/xdebug/vscode-php-debug) |
| Phpactor | The PHP Suite's language server | phpactor | MIT | [github.com/phpactor/phpactor](https://github.com/phpactor/phpactor) |
| Segment Anything (SAM ViT-B) | Picks out the parts of a wallpaper in the depth Suite | Meta AI Research | Apache-2.0 | [github.com/facebookresearch/segment-anything](https://github.com/facebookresearch/segment-anything) |
| vscode-js-debug | The Node Suite's debug adapter | Microsoft Corporation | MIT | [github.com/microsoft/vscode-js-debug](https://github.com/microsoft/vscode-js-debug) |

### Kubernetes apps of the catalog

The Apps catalog's Kubernetes tools (`apps/`), and the marks their icons carry.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| Helm | The catalog's Helm (`apps/helm/`) | The Helm Authors | Apache-2.0 | [github.com/helm/helm](https://github.com/helm/helm) |
| kubectx + kubens | The catalog's kubectx + kubens (`apps/kubectx/`) | Ahmet Alp Balkan and the kubectx contributors | Apache-2.0 | [github.com/ahmetb/kubectx](https://github.com/ahmetb/kubectx) |
| stern | The catalog's stern (`apps/stern/`) | The stern authors | Apache-2.0 | [github.com/stern/stern](https://github.com/stern/stern) |
| kind | The catalog's kind (`apps/kind/`), and its logo as kind's icon | The Kubernetes Authors | Apache-2.0 | [github.com/kubernetes-sigs/kind](https://github.com/kubernetes-sigs/kind) |
| minikube | The catalog's minikube (`apps/minikube/`), and its logo as minikube's icon | The Kubernetes Authors | Apache-2.0 | [github.com/kubernetes/minikube](https://github.com/kubernetes/minikube) |
| k3d | The catalog's k3d (`apps/k3d/`), and its logo as k3d's icon | Thorsten Klein and the k3d contributors | MIT | [github.com/k3d-io/k3d](https://github.com/k3d-io/k3d) |
| Kustomize | The catalog's Kustomize (`apps/kustomize/`), and its icon as Kustomize's | The Kubernetes Authors | Apache-2.0 | [github.com/kubernetes-sigs/kustomize](https://github.com/kubernetes-sigs/kustomize) |
| Argo CD | The catalog's Argo CD CLI (`apps/argocd/`) | The Argo Project Authors | Apache-2.0 | [github.com/argoproj/argo-cd](https://github.com/argoproj/argo-cd) |
| Flux | The catalog's Flux CLI (`apps/flux/`) | The Flux Authors | Apache-2.0 | [github.com/fluxcd/flux2](https://github.com/fluxcd/flux2) |
| Headlamp | The catalog's Headlamp (`apps/headlamp/`), and its icon as Headlamp's | The Kubernetes Authors | Apache-2.0 | [github.com/kubernetes-sigs/headlamp](https://github.com/kubernetes-sigs/headlamp) |
| Tilt | The catalog's Tilt (`apps/tilt/`), and its logo as Tilt's icon | Tilt Dev Inc. and the Tilt contributors | Apache-2.0 | [github.com/tilt-dev/tilt](https://github.com/tilt-dev/tilt) |
| Skaffold | The catalog's Skaffold (`apps/skaffold/`), and its mark as Skaffold's icon | Google LLC and the Skaffold contributors | Apache-2.0 | [github.com/GoogleContainerTools/skaffold](https://github.com/GoogleContainerTools/skaffold) |
| Helm's logo | Helm's icon (`apps/helm/helm.svg`), from Helm's website | Microsoft Corporation and the Helm Authors | MIT | [github.com/helm/helm-www](https://github.com/helm/helm-www) |
| Argo's mark | The Argo CD CLI's icon (`apps/argocd/argocd.svg`) | The Argo Project Authors | The Linux Foundation's trademark, used to identify the app | [github.com/argoproj/argoproj](https://github.com/argoproj/argoproj) |
| Flux's mark | The Flux CLI's icon (`apps/flux/flux.svg`), from Flux's website | The Flux Authors | Apache-2.0 | [github.com/fluxcd/website](https://github.com/fluxcd/website) |

### Cloud, container and CI apps of the catalog

The Apps catalog's cloud CLIs, container and CI tools (`apps/`), and the marks their icons carry.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| AWS CLI | The catalog's AWS CLI (`apps/aws-cli/`) | Amazon.com, Inc. | Apache-2.0 | [github.com/aws/aws-cli](https://github.com/aws/aws-cli) |
| Google Cloud CLI | The catalog's Google Cloud CLI (`apps/google-cloud-cli/`) | Google LLC | Apache-2.0 | [cloud.google.com/sdk](https://cloud.google.com/sdk) |
| Azure CLI | The catalog's Azure CLI (`apps/azure-cli/`) | Microsoft Corporation | MIT | [github.com/Azure/azure-cli](https://github.com/Azure/azure-cli) |
| doctl | The catalog's doctl (`apps/doctl/`) | DigitalOcean, LLC | Apache-2.0 | [github.com/digitalocean/doctl](https://github.com/digitalocean/doctl) |
| hcloud | The catalog's hcloud (`apps/hcloud/`) | Hetzner Cloud GmbH | MIT | [github.com/hetznercloud/cli](https://github.com/hetznercloud/cli) |
| flyctl | The catalog's flyctl (`apps/flyctl/`) | Fly.io, Inc. | Apache-2.0 | [github.com/superfly/flyctl](https://github.com/superfly/flyctl) |
| dive | The catalog's dive (`apps/dive/`) | Alex Goodman | MIT | [github.com/wagoodman/dive](https://github.com/wagoodman/dive) |
| act | The catalog's act (`apps/act/`), and its logo as act's icon | The act authors | MIT | [github.com/nektos/act](https://github.com/nektos/act) |
| GitLab CLI | The catalog's GitLab CLI (`apps/glab/`) | GitLab Inc. | MIT | [gitlab.com/gitlab-org/cli](https://gitlab.com/gitlab-org/cli) |
| The Amazon Web Services logo | AWS CLI's icon (`apps/aws-cli/aws.svg`) | Amazon.com, Inc. | Amazon's trademark, used to identify the app | [aws.amazon.com/trademark-guidelines](https://aws.amazon.com/trademark-guidelines/) |
| The Google Cloud icon | Google Cloud CLI's icon (`apps/google-cloud-cli/google-cloud.png`) | Google LLC | Google's trademark, used to identify the app | [cloud.google.com](https://cloud.google.com/) |
| The Microsoft Azure icon | Azure CLI's icon (`apps/azure-cli/azure.svg`) | Microsoft Corporation | Microsoft's trademark, used to identify the app | [azure.microsoft.com](https://azure.microsoft.com/) |
| DigitalOcean's mark | doctl's icon (`apps/doctl/digitalocean.svg`) | DigitalOcean, LLC | DigitalOcean's trademark, used to identify the app | [digitalocean.com](https://www.digitalocean.com/) |
| Hetzner's logo | hcloud's icon (`apps/hcloud/hetzner.png`) | Hetzner Online GmbH | Hetzner's trademark, used to identify the app | [hetzner.com](https://www.hetzner.com/) |
| GitLab's tanuki | GitLab CLI's icon (`apps/glab/gitlab.svg`) | GitLab Inc. | GitLab's trademark, used to identify the app | [gitlab.com/gitlab-org/gitlab](https://gitlab.com/gitlab-org/gitlab) |

### Infrastructure-as-code, secrets and security apps of the catalog

The Apps catalog's infrastructure-as-code, secrets and security tools (`apps/`), and the marks their icons carry.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| OpenTofu | The catalog's OpenTofu, from its project's apt repository | The OpenTofu Authors, a Linux Foundation project | MPL-2.0 | [github.com/opentofu/opentofu](https://github.com/opentofu/opentofu) |
| OpenTofu's brand artifacts | OpenTofu's catalog icon, its logo mark | The OpenTofu Authors | MPL-2.0; the trademark under the Linux Foundation's trademark policy | [github.com/opentofu/brand-artifacts](https://github.com/opentofu/brand-artifacts) |
| Terraform | The catalog's Terraform, from HashiCorp's apt repository | HashiCorp, an IBM company | BUSL-1.1 | [github.com/hashicorp/terraform](https://github.com/hashicorp/terraform) |
| Packer | The catalog's Packer, from HashiCorp's apt repository | HashiCorp, an IBM company | BUSL-1.1 | [github.com/hashicorp/packer](https://github.com/hashicorp/packer) |
| Vault | The catalog's Vault CLI, from HashiCorp's apt repository | HashiCorp, an IBM company | BUSL-1.1 | [github.com/hashicorp/vault](https://github.com/hashicorp/vault) |
| Flight Icons | Terraform's, Packer's and Vault's catalog icons, their logos; HashiCorp's trademarks | HashiCorp, Inc | MPL-2.0 | [github.com/hashicorp/design-system](https://github.com/hashicorp/design-system/tree/main/packages/flight-icons) |
| Pulumi | The catalog's Pulumi, its release's archive | Pulumi Corporation | Apache-2.0 | [github.com/pulumi/pulumi](https://github.com/pulumi/pulumi) |
| Pulumi's docs | Pulumi's catalog icon, its mark; Pulumi's trademark | Pulumi Corporation | Apache-2.0 | [github.com/pulumi/docs](https://github.com/pulumi/docs) |
| Ansible | The catalog's Ansible, from Ubuntu's archive | Red Hat and the Ansible community | GPL-3.0-or-later | [github.com/ansible/ansible](https://github.com/ansible/ansible) |
| The Ansible mark | Ansible's catalog icon; Red Hat's trademark | Red Hat, Inc. | Trademark, used to identify the app | [docs.ansible.com](https://docs.ansible.com/images/community_logo_black.svg) |
| SOPS | The catalog's SOPS, from nixpkgs | The SOPS Authors, a CNCF project | MPL-2.0 | [github.com/getsops/sops](https://github.com/getsops/sops) |
| SOPS's docs | SOPS's catalog icon, its logo | The SOPS Authors | CC-BY-4.0 | [github.com/getsops/docs](https://github.com/getsops/docs) |
| age | The catalog's age, from nixpkgs | Filippo Valsorda and Ben Cartwright-Cox, the age Authors | BSD-3-Clause | [github.com/FiloSottile/age](https://github.com/FiloSottile/age) |
| Trivy | The catalog's Trivy, from Aqua Security's apt repository; its catalog icon, its logo mark | Aqua Security Software Ltd. | Apache-2.0; its logo CC-BY-4.0 | [github.com/aquasecurity/trivy](https://github.com/aquasecurity/trivy) |

### Rust crates

What the workspace's crates depend on directly.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| `async-channel` | The Bluetooth agent's queues between its D-Bus side and its waits | Stjepan Glavina | Apache-2.0 OR MIT | [github.com/smol-rs/async-channel](https://github.com/smol-rs/async-channel) |
| `async-io` | The event loop zbus runs on in the automount and the Bluetooth agent | Stjepan Glavina | Apache-2.0 OR MIT | [github.com/smol-rs/async-io](https://github.com/smol-rs/async-io) |
| `base64` | Encodes the Apps catalog actions' file contents | Marshall Pierce | MIT OR Apache-2.0 | [github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `clap` | The `kubaos` command's arguments | clap's individual contributors | MIT OR Apache-2.0 | [github.com/clap-rs/clap](https://github.com/clap-rs/clap) |
| `crc32fast` | Checksums in the cursor theme's archive | Sam Rijs and Alex Crichton | MIT OR Apache-2.0 | [github.com/srijs/rust-crc32fast](https://github.com/srijs/rust-crc32fast) |
| `flate2` | Unpacks apt's package lists in `kubaos-apps-verify` | Alex Crichton and Josh Triplett | MIT OR Apache-2.0 | [github.com/rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs) |
| `futures-lite` | Waits on D-Bus signals in the automount and the Bluetooth agent | Stjepan Glavina and contributors to futures-rs | Apache-2.0 OR MIT | [github.com/smol-rs/futures-lite](https://github.com/smol-rs/futures-lite) |
| `iced` | The toolkit of `kubaos-ui`, Kōbō and the specimen | Héctor Ramón Jiménez | MIT | [github.com/iced-rs/iced](https://github.com/iced-rs/iced) |
| `iced_runtime` | Draws screens without a window, for the screenshots and the geometry check | Héctor Ramón Jiménez | MIT | [github.com/iced-rs/iced](https://github.com/iced-rs/iced) |
| `iced_test` | Lays views out headlessly in the tests | Héctor Ramón Jiménez | MIT | [github.com/iced-rs/iced](https://github.com/iced-rs/iced) |
| `iced_tiny_skia` (patched copy in `vendor/`) | iced's software renderer, patched to place and clip images exactly | Héctor Ramón Jiménez | MIT | [github.com/iced-rs/iced](https://github.com/iced-rs/iced) |
| `image` | Reads and writes the wallpapers' depth layers and Kōbō's pictures | The image-rs Developers | MIT OR Apache-2.0 | [github.com/image-rs/image](https://github.com/image-rs/image) |
| `jiff` | Dates, times and zones for the clock, the counters, idle and power | Andrew Gallant | Unlicense OR MIT | [github.com/BurntSushi/jiff](https://github.com/BurntSushi/jiff) |
| `kamadak-exif` | The photo import's capture dates | KAMADA Ken'ichi | BSD-2-Clause | [github.com/kamadak/exif-rs](https://github.com/kamadak/exif-rs) |
| `kotodama-extension` | The KubaOS Extension's side of the Command menu | SuplenC | Apache-2.0 | [github.com/SuplenC/kotodama](https://github.com/SuplenC/kotodama) |
| `kotodama-protocol` | The messages between the Extension and the menu | SuplenC | Apache-2.0 | [github.com/SuplenC/kotodama](https://github.com/SuplenC/kotodama) |
| `kotodamad` | Ranks the Extension's Items in its tests as the menu does | SuplenC | Apache-2.0 | [github.com/SuplenC/kotodama](https://github.com/SuplenC/kotodama) |
| `log` | The services' logs | The Rust Project Developers | MIT OR Apache-2.0 | [github.com/rust-lang/log](https://github.com/rust-lang/log) |
| `ort` | ONNX Runtime from Rust, for the depth models | pyke.io and Nicolas Bigaouette | MIT OR Apache-2.0 | [github.com/pykeio/ort](https://github.com/pykeio/ort) |
| `png` | Writes the specimen's screenshots | The image-rs Developers | MIT OR Apache-2.0 | [github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| `resvg` | Renders Bibata's SVG cursors in the wallpaper's colour | The Resvg Authors | Apache-2.0 OR MIT | [github.com/linebender/resvg](https://github.com/linebender/resvg) |
| `rustix` | System calls without `unsafe`: priorities, terminals, files | Dan Gohman and Jakub Konka | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| `serde` | Reads and writes every settings and state file | Erick Tryzelaar and David Tolnay | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_json` | JSON for the engine's events, the CLI and the geometry dumps | Erick Tryzelaar and David Tolnay | MIT OR Apache-2.0 | [github.com/serde-rs/json](https://github.com/serde-rs/json) |
| `sha2` | Checks pinned downloads' sha256 in `kubaos-apps-verify` | RustCrypto Developers | MIT OR Apache-2.0 | [github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| `signal-hook` | Stops the CLI and Kōbō cleanly on a signal | Michal 'vorner' Vaner and Thomas Himmelstoss | Apache-2.0/MIT | [github.com/vorner/signal-hook](https://github.com/vorner/signal-hook) |
| `tempfile` | Atomic writes and the tests' scratch folders | Steven Allen, Ashley Mannix, Jason White and The Rust Project Developers | MIT OR Apache-2.0 | [github.com/Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| `thiserror` | The crates' error types | David Tolnay | MIT OR Apache-2.0 | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `tiny-skia` | Rasterises Kōbō's hanko | Yevhenii Reizner | BSD-3-Clause | [github.com/RazrFalcon/tiny-skia](https://github.com/RazrFalcon/tiny-skia) |
| `toml` | Reads the settings and the manifests | toml-rs's individual contributors | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | Writes settings in place, keeping the user's comments | toml-rs's individual contributors | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `unicode-segmentation` | A name's first letter for the icon tiles | kwantam and Manish Goregaokar | MIT OR Apache-2.0 | [github.com/unicode-rs/unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) |
| `zbus` | D-Bus: udisks2, BlueZ, NetworkManager, UPower and systemd | Zeeshan Ali Khan | MIT | [github.com/z-galaxy/zbus](https://github.com/z-galaxy/zbus/) |

### Mockups

What the mockups (`design/mockups/`) ship to the page.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| `atmos-fx` | Rain, snow and hail over the weather's skies (`design/mockups/src/lib/weather-sky.ts`) | Carson Ye and contributors | MIT | [github.com/Whynotmetoo/atmos-fx](https://github.com/Whynotmetoo/atmos-fx) |

### Fonts, icons, cursors and sounds

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| Bibata | The pointer `kubaos cursor` recolours from the wallpaper | Abdulkaiz Khatri | GPL-3.0-only | [github.com/ful1e5/Bibata_Cursor](https://github.com/ful1e5/Bibata_Cursor) |
| Caskaydia Cove (Cascadia Code, Nerd Fonts build) | A terminal font to pick | Aaron Bell (Saja Typeworks) for Microsoft Corporation | OFL-1.1 | [nerdfonts.com](https://nerdfonts.com/) |
| Colloid icon theme | The apps' icons, in its orange variant | vinceliuice | GPL-3.0-only | [github.com/vinceliuice/colloid-icon-theme](https://github.com/vinceliuice/colloid-icon-theme) |
| DejaVu fonts | A fallback font in the screenshot harness | Bitstream, Inc., Tavmjong Bah and the DejaVu fonts team | Bitstream Vera licence; DejaVu changes public domain | [dejavu-fonts.github.io](https://dejavu-fonts.github.io/) |
| Geist and Geist Mono | The interface and monospace fonts, and a generated app icon's initial, outlined ([`Geist-OFL.txt`](crates/kubaos-ui/assets/fonts/Geist-OFL.txt)) | The Geist Project Authors: Basement.studio, Vercel, Andrés Briganti, Guido Ferreyra and Mateo Zaragoza | OFL-1.1 | [github.com/vercel/geist-font](https://github.com/vercel/geist-font) |
| JetBrains Mono (Nerd Fonts build) | A terminal font to pick | Philipp Nurullin and Konstantin Bulenkov (JetBrains) | OFL-1.1 | [nerdfonts.com](https://nerdfonts.com/) |
| Lucide | One of the icon sets the specimen compares ([licence](crates/kubaos-specimen/assets/icons/lucide-LICENSE.txt)) | Lucide Icons and Contributors | ISC | [npmjs.com/package/lucide-static](https://www.npmjs.com/package/lucide-static) |
| M PLUS 1 | The Japanese accents: the weekday kanji, 曜日 and 東京, and a generated app icon's kana or kanji, outlined ([`MPLUS1-OFL.txt`](crates/kubaos-ui/assets/fonts/MPLUS1-OFL.txt)) | The M+ FONTS Project Authors: Coji Morishita | OFL-1.1 | [github.com/coz-m/MPLUS_FONTS](https://github.com/coz-m/MPLUS_FONTS) |
| Material Symbols | One of the icon sets the specimen compares ([licence](crates/kubaos-specimen/assets/icons/material-LICENSE.txt)) | Google | Apache-2.0 | [github.com/google/material-design-icons](https://github.com/google/material-design-icons) |
| Nerd Fonts | Patches the terminal fonts with icons | Ryan L McIntyre | MIT for the patcher; each font keeps its own licence | [github.com/ryanoasis/nerd-fonts](https://github.com/ryanoasis/nerd-fonts) |
| Noto | The system's fallback fonts, CJK and colour emoji included | Google | OFL-1.1 | [google.com/get/noto](https://www.google.com/get/noto/) |
| Phosphor | One of the icon sets the specimen compares ([licence](crates/kubaos-specimen/assets/icons/phosphor-LICENSE.txt)) | Phosphor Icons | MIT | [npmjs.com/package/@phosphor-icons/web](https://www.npmjs.com/package/@phosphor-icons/web) |
| sound-theme-freedesktop | The sound the clock's timers and alarms ring with, and the power warning's | The freedesktop.org sound theme's authors, named per sound in its CREDITS file | Mixed, per sound: CC-BY-SA, CC-BY-3.0, GPL-2.0-or-later, GPL-2.0 | [cgit.freedesktop.org/sound-theme-freedesktop](https://cgit.freedesktop.org/sound-theme-freedesktop) |
| Tabler Icons | The KubaOS Symbols set (`crates/kubaos-ui/assets/symbols/`), a generated app icon's symbol, and the style its weather symbols are drawn in ([`TABLER-LICENSE.txt`](crates/kubaos-ui/assets/symbols/TABLER-LICENSE.txt)) | Paweł Kuna | MIT | [github.com/tabler/tabler-icons](https://github.com/tabler/tabler-icons) |

### Services

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| Android Studio's release list | Where the Apps bump finds Android Studio's newest stable release | JetBrains s.r.o. | Its terms of service | [jb.gg/android-studio-releases-list.json](https://jb.gg/android-studio-releases-list.json) |
| Blender's download server | Where the Apps bump finds Blender's newest release, from its folder listing | Blender Foundation | Its terms of service | [download.blender.org/release](https://download.blender.org/release/) |
| Cachix | The `kuba-os` binary cache that CI fills and `kubaos update` reads | Cachix | Its terms of service | [kuba-os.cachix.org](https://kuba-os.cachix.org) |
| GitHub | Hosts KubaOS and Kotodama, runs CI (Actions) and the mockup site (Pages); its releases API finds `kubaos update`'s changelog, the Bitwarden CLI's releases and the Apps bump's newest releases | GitHub, Inc. | Its terms of service | [github.com](https://github.com) |
| Hugging Face | Serves the Depth Anything V2 Small model the depth Suite downloads | Hugging Face | Its terms of service | [huggingface.co](https://huggingface.co) |
| JetBrains' release API | Where the Apps bump finds JetBrains Toolbox's newest release | JetBrains s.r.o. | Its terms of service | [data.services.jetbrains.com](https://data.services.jetbrains.com/products/releases?code=TBA&latest=true&type=release) |
| Open-Meteo | The forecast source the clock's weather is designed around; the clock popup's "Weather from Open-Meteo" is the attribution its data licence asks for | Open-Meteo | The API's code AGPL-3.0-or-later; the weather data CC-BY-4.0 | [open-meteo.com](https://open-meteo.com) |
| Proton Pass's release feed | Where the Apps bump finds Proton Pass's newest release out to everyone | Proton AG | Its terms of service | [proton.me/download/PassDesktop/linux/x64/version.json](https://proton.me/download/PassDesktop/linux/x64/version.json) |

### Credits kept beside the files they cover

- [`design/mockups/src/assets/audio/CREDITS.md`](design/mockups/src/assets/audio/CREDITS.md): the media player mockup's songs.
- [`design/mockups/src/assets/lock/CREDITS.md`](design/mockups/src/assets/lock/CREDITS.md): the Lock mockup's wallpapers and the tools that cut their depth.
- The mockups' app icons (`design/mockups/src/assets/apps/`): each names its source and licence in its header comment.
- [`branding/README.md`](branding/README.md): how the logo's photos were made, and their licence.
- [`LICENSES/`](LICENSES): the licence texts the third-party files under `apps/*/` name in their headers.
- The wallpapers' credits: [SuplenC/kubaos-wallpapers](https://github.com/SuplenC/kubaos-wallpapers), each set's `CREDITS.md`.
- The Apps catalog's apps belong to their makers, each named in its `app.toml`; their icons and files name their own sources and licences in their headers, with the texts in [`LICENSES/`](LICENSES).

## What it is made with

The development tools KubaOS is made with.

### Build and test

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| actionlint | Lints the CI workflows | rhysd | MIT | [rhysd.github.io/actionlint](https://rhysd.github.io/actionlint/) |
| bubblewrap | Sandboxes the screenshot harness's programs | containers | LGPL-2.1-or-later | [github.com/containers/bubblewrap](https://github.com/containers/bubblewrap) |
| Cargo | Builds and tests the workspace | The Rust Project Contributors | MIT, Apache-2.0 | [crates.io](https://crates.io) |
| cargo-nextest | Runs the workspace's tests (`just nextest`) | The nextest Contributors | MIT, Apache-2.0 | [github.com/nextest-rs/nextest](https://github.com/nextest-rs/nextest) |
| Chromium (Chrome for Testing) | The browser Playwright drives for the mockups' shots and checks | The Chromium Authors | BSD-3-Clause | [chromium.org](https://www.chromium.org) |
| Clippy | The Rust lints (`just lint`) | The Rust Project Contributors | MIT, Apache-2.0 | [rust-lang.github.io/rust-clippy](https://rust-lang.github.io/rust-clippy/) |
| crane | Builds the crates with Nix (`crates/build.nix`) | Ivan Petkov | MIT | [github.com/ipetkov/crane](https://github.com/ipetkov/crane) |
| curl | Downloads in the scripts and the dev shell | Daniel Stenberg and contributors | curl | [curl.se](https://curl.se/) |
| Docker Engine | The end-to-end test containers (`test/run`) and the install image's build | Docker, Inc. | Apache-2.0 | [github.com/moby/moby](https://github.com/moby/moby) |
| fontconfig | Font lookup in the screenshot harness | Keith Packard | BSD-2-Clause | [fontconfig.org](http://fontconfig.org/) |
| fontTools | Outlines a generated app icon's initial from Geist or M PLUS 1 (`design/app-icon.py`) | Just van Rossum, Behdad Esfahbod and the fontTools authors | MIT | [github.com/fonttools/fonttools](https://github.com/fonttools/fonttools) |
| GnuPG | Checks vendor keys in the dev shell and the system scripts | Free Software Foundation, Inc. | GPL-3.0-or-later | [gnupg.org](https://gnupg.org) |
| hyperfine | Benchmarks commands | David Peter | Apache-2.0, MIT | [github.com/sharkdp/hyperfine](https://github.com/sharkdp/hyperfine) |
| just | Runs the repo's recipes (`justfile`) | Casey Rodarmor | CC0-1.0 | [github.com/casey/just](https://github.com/casey/just) |
| Mesa | Software rendering for the screenshot harness | The Mesa authors | MIT for the core, other licences per file | [docs.mesa3d.org/license.html](https://docs.mesa3d.org/license.html) |
| nixfmt | Formats the `.nix` files | NixOS | MPL-2.0 | [github.com/NixOS/nixfmt](https://github.com/NixOS/nixfmt) |
| Node.js | Runs the mockups' build and scripts | Node.js contributors | MIT | [nodejs.org](https://nodejs.org) |
| NumPy | The neon sign's switch-on frames (`branding/switch.py`) | Travis E. Oliphant and the NumPy Developers | BSD-3-Clause | [numpy.org](https://numpy.org/) |
| OVMF | UEFI firmware for the install's VM tests | tianocore | BSD-2-Clause | [github.com/tianocore/tianocore.github.io/wiki/OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF) |
| Pillow | Draws the branding and checks the screenshots | Jeffrey 'Alex' Clark and contributors | MIT-CMU | [python-pillow.github.io](https://python-pillow.github.io/) |
| Playwright (`playwright-core`) | Drives the headless browser that renders the mockups' shots and checks | Microsoft Corporation | Apache-2.0 | [playwright.dev](https://playwright.dev) |
| Python | The branding scripts and the screenshot harness | The Python Software Foundation | Python-2.0 | [python.org](https://www.python.org) |
| QEMU | Boots the install images in the VM tests | Fabrice Bellard and the QEMU Project Developers | GPL-2.0-or-later | [qemu.org](https://www.qemu.org/) |
| Rust | The language of the engine, the CLI, the services and the apps | The Rust Project Contributors | MIT, Apache-2.0 | [rust-lang.org](https://www.rust-lang.org/) |
| rustfmt | Formats the Rust code (nightly, `just fmt-check`) | The Rust Project Developers | MIT, Apache-2.0 | [github.com/rust-lang-nursery/rustfmt](https://github.com/rust-lang-nursery/rustfmt) |
| sccache | Caches Rust compiles in the dev shell and CI | mozilla | Apache-2.0 | [github.com/mozilla/sccache](https://github.com/mozilla/sccache) |
| ShellCheck | Lints the shell scripts (`just shellcheck`), and Neovim's shell files through Mason | Vidar Holen and contributors | GPL-3.0 | [github.com/koalaman/shellcheck](https://github.com/koalaman/shellcheck) |
| Sway | The headless compositor the screenshot harness renders the shell in | Simon Ser and the Sway contributors | MIT | [swaywm.org](https://swaywm.org) |
| treefmt | Runs the formatters behind `nix fmt` | Numtide and contributors | MIT | [github.com/numtide/treefmt](https://github.com/numtide/treefmt) |
| `@types/node` | Node's types for the mockups' scripts | Microsoft Corporation and the DefinitelyTyped contributors | MIT | [github.com/DefinitelyTyped/DefinitelyTyped](https://github.com/DefinitelyTyped/DefinitelyTyped/tree/master/types/node) |
| TypeScript (`typescript`) | The mockups' language and their type check | Microsoft Corporation | Apache-2.0 | [github.com/microsoft/TypeScript](https://github.com/microsoft/TypeScript) |
| Vite (`vite`) | Serves and builds the mockups (`design/mockups/`) | Evan You, VoidZero Inc. and Vite contributors | MIT | [vite.dev](https://vite.dev) |
| xdotool | Drives X apps in the screenshot harness | Jordan Sissel | BSD-3-Clause | [semicomplete.com/projects/xdotool](https://www.semicomplete.com/projects/xdotool/) |
| xorriso | Builds the install ISO and the VM tests' boot media | Thomas Schmitt; libisoburn by Vreixo Formoso and Thomas Schmitt | GPL-2.0-or-later | [libburnia-project.org](http://libburnia-project.org/) |
| Xvfb | The virtual display the boot screens render in | The X.Org contributors, from Digital Equipment Corporation's sample server | MIT and others | [x.org/wiki](https://x.org/wiki) |

### Design and branding

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| FLUX 3 | Generated the logo's photos (`branding/photo/`) | Black Forest Labs | Its model licence; see its site | [bfl.ai](https://bfl.ai) |

### CI

The actions `.github/workflows/` runs, pinned by commit.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| `actions/cache` | Restores and saves the compile caches | GitHub, Inc. | MIT | [github.com/actions/cache](https://github.com/actions/cache) |
| `actions/checkout` | Checks out the repository | GitHub, Inc. | MIT | [github.com/actions/checkout](https://github.com/actions/checkout) |
| `actions/deploy-pages` | Publishes the mockup site | GitHub, Inc. | MIT | [github.com/actions/deploy-pages](https://github.com/actions/deploy-pages) |
| `actions/download-artifact` | Carries the screenshots between jobs | GitHub, Inc. | MIT | [github.com/actions/download-artifact](https://github.com/actions/download-artifact) |
| `actions/upload-artifact` | Keeps the screenshots and the mockups' build | GitHub, Inc. | MIT | [github.com/actions/upload-artifact](https://github.com/actions/upload-artifact) |
| `actions/upload-pages-artifact` | Packs the mockup site for GitHub Pages | GitHub, Inc. | MIT | [github.com/actions/upload-pages-artifact](https://github.com/actions/upload-pages-artifact) |
| `cachix/cachix-action` | Reads the `kuba-os` binary cache | Cachix | Apache-2.0 | [github.com/cachix/cachix-action](https://github.com/cachix/cachix-action) |
| `cachix/install-nix-action` | Installs Nix on the runners | Cachix | See its repository | [github.com/cachix/install-nix-action](https://github.com/cachix/install-nix-action) |
| `nix-community/cache-nix-action` | Restores and saves the Nix store | nix-community | See its repository | [github.com/nix-community/cache-nix-action](https://github.com/nix-community/cache-nix-action) |
