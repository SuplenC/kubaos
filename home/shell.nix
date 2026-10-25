# zsh, ported from the macOS ~/.zshrc and ~/.zprofile.
#
# programs.zsh writes ~/.zshrc, ~/.zshenv and ~/.zprofile. Order of
# programs.zsh.initContent (lower runs first):
#   550   the init cache (_kubaos_init, below)
#   800   oh-my-zsh (plugins: git, no theme), compinit from a cached dump
#   900   FZF_CTRL_R_COMMAND="" (keeps fzf off Ctrl-R)
#   910   fzf (Ctrl-T, Alt-C)
#   1000  aliases, oh-my-posh
#   1010  direnv hook, atuin (Ctrl-R)
#   1900  the user layer's zshrc (~/.config/kubaos/terminal/zsh/zshrc)
#   1950  the greeting: fastfetch in a new Ghostty window
#   2000  zoxide as `cd`, last, so `zoxide doctor` stays quiet
#
# The harness's zsh_start check fails when an interactive start goes over its
# budget. The tools' init scripts and the completion dump are cached in
# ~/.cache/kubaos/zsh; deleting that folder is always safe.
#
# PATH for proto/bun/cargo/~/.local/bin lives in home.sessionPath (nix agent).
{
  config,
  pkgs,
  lib,
  ...
}:
let
  userLayer = import ./user-layer.nix { configHome = config.xdg.configHome; };
  # The apps' folders relative to the user layer: zsh reads them under
  # $XDG_CONFIG_HOME at run time.
  inherit (userLayer) apps;

  # A tool's `init zsh` output is cached in a file named after a hash of the
  # command, store path included, so an update or new flags write a new file
  # and the old one is never read again.
  cachedInit =
    name: argv:
    let
      command = lib.escapeShellArgs argv;
      key = builtins.substring 0 16 (builtins.hashString "sha256" command);
    in
    "_kubaos_init ${name}-${key} ${command}";

  omp = lib.getExe pkgs.oh-my-posh;
  ompKey = builtins.substring 0 16 (builtins.hashString "sha256" omp);
in
{
  home.packages = [ pkgs.oh-my-posh ];

  # XDG_CONFIG_HOME and friends come from xdg.enable (default.nix).
  home.sessionVariables.EDITOR = "nvim";

  programs.zsh = {
    enable = true;
    # Keep ~/.zshrc in $HOME (newer home-manager defaults to ~/.config/zsh).
    dotDir = config.home.homeDirectory;

    autosuggestion.enable = true;

    oh-my-zsh = {
      enable = true;
      plugins = [ "git" ];
      theme = ""; # oh-my-posh draws the prompt
      # oh-my-zsh audits every $fpath folder and runs a full compinit at each
      # start: 12 ms of the 51 ms an interactive start took in the harness.
      # The audit is off, and the `compinit` oh-my-zsh calls is this
      # function: it trusts the dump (-C) and rebuilds it once a day.
      # oh-my-zsh deletes the dump itself when $fpath changes, so a new
      # completion folder shows at the next start. `autoload -U compinit` in
      # oh-my-zsh.sh leaves an already defined function alone.
      extraConfig = ''
        ZSH_DISABLE_COMPFIX=true
        ZSH_COMPDUMP="$_kubaos_zsh_cache/zcompdump-$ZSH_VERSION"
        function compinit() {
          unfunction compinit
          autoload -Uz compinit
          local -a fresh=($ZSH_COMPDUMP(N.mh-24))
          if (( $#fresh )); then
            compinit -C -d "$ZSH_COMPDUMP"
            return
          fi
          [[ -d $_kubaos_zsh_cache ]] || mkdir -p -- "$_kubaos_zsh_cache"
          rm -f -- "$ZSH_COMPDUMP" "$ZSH_COMPDUMP.zwc"
          compinit -i -d "$ZSH_COMPDUMP"
        }
      '';
    };

    shellAliases = {
      zshrc = "nvim ${userLayer.dir "zsh"}/zshrc";
      wipe = "clear && tmux clear-history";
      nvimconfig = "nvim ~/.config/nvim";
      tma = "tmux attach";
    };

    initContent = lib.mkMerge [
      # Forking oh-my-posh, atuin, fzf, direnv and zoxide for their init
      # scripts cost 7 ms of each start in the harness. `_kubaos_init NAME
      # CMD...` sources NAME's cached output of CMD, and runs CMD to write it
      # when the file is missing or empty.
      (lib.mkOrder 550 ''
        typeset -g _kubaos_zsh_cache="''${XDG_CACHE_HOME:-$HOME/.cache}/kubaos/zsh"
        function _kubaos_init() {
          local file="$_kubaos_zsh_cache/$1.zsh"
          shift
          if [[ ! -s $file ]]; then
            [[ -d $_kubaos_zsh_cache ]] || mkdir -p -- "$_kubaos_zsh_cache"
            # HAZARD: write a temporary file and rename it. Two shells
            # starting at once would otherwise source each other's half.
            if ! "$@" >|"$file.$$"; then
              rm -f -- "$file.$$"
              eval "$("$@")"
              return
            fi
            mv -f -- "$file.$$" "$file"
          fi
          source "$file"
        }
      '')

      # Keeps fzf off Ctrl-R (atuin owns it). home-manager 26.05 has no option
      # for this; fzf's key-bindings.zsh skips Ctrl-R when it is set but empty.
      (lib.mkOrder 900 ''
        export FZF_CTRL_R_COMMAND=""
      '')

      # Where fzf's own module would run `fzf --zsh`.
      (lib.mkOrder 910 ''
        if [[ $options[zle] = on ]]; then
          ${cachedInit "fzf" [
            (lib.getExe config.programs.fzf.package)
            "--zsh"
          ]}
        fi
      '')

      ''
        ulimit -n 65536 2>/dev/null

        function y() {
          local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
          command yazi "$@" --cwd-file="$tmp"
          IFS= read -r -d ''' cwd < "$tmp"
          [ "$cwd" != "$PWD" ] && [ -d "$cwd" ] && builtin cd -- "$cwd"
          command rm -f -- "$tmp"
        }

        # Wallpaper theme files from dot-theme (~/.local/state/theme). Each
        # tool reads its file at start; only set the variables when the file
        # exists, because lazygit refuses to start with a missing config.
        _dot_theme="$HOME/.local/state/theme"
        [[ -r $_dot_theme/fzf ]] && export FZF_DEFAULT_OPTS_FILE="$_dot_theme/fzf"
        # The user layer (~/.config/kubaos, docs/CONTRACTS.md) goes last, so it wins.
        _dot_user="''${XDG_CONFIG_HOME:-$HOME/.config}/kubaos"
        () {
          local -a lg=("''${XDG_CONFIG_HOME:-$HOME/.config}/lazygit/config.yml")
          [[ -r $_dot_theme/lazygit.yml ]] && lg+=("$_dot_theme/lazygit.yml")
          [[ -r $_dot_user/${apps.lazygit}/config.yml ]] && lg+=("$_dot_user/${apps.lazygit}/config.yml")
          (( $#lg > 1 )) && export LG_CONFIG_FILE="''${(j:,:)lg}"
        }
        [[ -r $_dot_user/${apps.bacon}/prefs.toml ]] && export BACON_PREFS="$_dot_user/${apps.bacon}/prefs.toml"
        # oh-my-zsh exports LS_COLORS (dircolors) and eza lets it override
        # ~/.config/eza/theme.yml; keep LS_COLORS for ls, fd and completion.
        function eza() { LS_COLORS= command eza "$@"; }
        function glow() {
          local style="$HOME/.local/state/theme/glow.json"
          if [[ -r $style ]]; then command glow -s "$style" "$@"; else command glow "$@"; fi
        }

        # oh-my-posh: themed config from dot-theme if present, else the static
        # one. `dot-theme set` rewrites ohmyposh.toml; _dot_omp_init runs
        # before each prompt and re-inits oh-my-posh when the file's mtime
        # changed, so open shells pick up the new colors at their next prompt
        # (one stat per prompt, no fork). The init script is cached by
        # oh-my-posh store path, config file and mtime, less the line that
        # exports the session id, which must differ from shell to shell.
        zmodload -F zsh/stat b:zstat
        typeset -g _dot_omp_mtime=""
        function _dot_omp_init() {
          local cfg="$_dot_theme/ohmyposh.toml"
          local -a st
          [[ -r $cfg ]] || cfg="$HOME/.config/ohmyposh/config.toml"
          zstat -A st +mtime -- "$cfg" 2>/dev/null || st=(0)
          [[ "$cfg:$st[1]" == "$_dot_omp_mtime" ]] && return
          _dot_omp_mtime="$cfg:$st[1]"
          local file="$_kubaos_zsh_cache/omp-${ompKey}-''${cfg:t:r}-$st[1].zsh"
          if [[ ! -s $file ]] && ! _dot_omp_cache "$cfg" "$file"; then
            eval "$(${omp} init zsh --config "$cfg")"
            return
          fi
          export POSH_SESSION_ID="$(</proc/sys/kernel/random/uuid)"
          source "$file"
        }
        # _dot_omp_cache CONFIG FILE: write oh-my-posh's init script for
        # CONFIG to FILE, replacing the one for the config's older mtime.
        function _dot_omp_cache() {
          setopt local_options pipefail
          [[ -d $_kubaos_zsh_cache ]] || mkdir -p -- "$_kubaos_zsh_cache"
          rm -f -- "$_kubaos_zsh_cache"/omp-*-''${1:t:r}-*.zsh(N)
          ${omp} init zsh --print --config "$1" | sed '/^export POSH_SESSION_ID=/d' >|"$2.$$" &&
            mv -f -- "$2.$$" "$2" || { rm -f -- "$2.$$"; return 1; }
        }
        _dot_omp_init
        precmd_functions=(_dot_omp_init ''${precmd_functions:#_dot_omp_init})
      ''

      # Where the direnv and atuin modules put their hooks.
      (lib.mkOrder 1010 ''
        ${cachedInit "direnv" [
          (lib.getExe config.programs.direnv.package)
          "hook"
          "zsh"
        ]}
        if [[ $options[zle] = on ]]; then
          ${cachedInit "atuin" (
            [
              (lib.getExe config.programs.atuin.package)
              "init"
              "zsh"
            ]
            ++ config.programs.atuin.flags
          )}
        fi
      '')

      # The user layer's zshrc, after everything but the greeting and zoxide,
      # so it wins.
      (lib.mkOrder 1900 ''
        [[ -r $_dot_user/${apps.zsh}/zshrc ]] && source "$_dot_user/${apps.zsh}/zshrc"
      '')

      # The greeting: fastfetch with the logo in the shell a new Ghostty
      # window opens. Never in tmux, Neovim, SSH, `zsh -c`, a shell started
      # from another, or when the output isn't a terminal; a `ghostty -e`
      # window runs its command, not this shell. SHLVL can't tell a nested
      # shell here: Hyprland's environment, which systemd hands on to the
      # Ghostty service, already holds SHLVL=1. So the greeting exports
      # KUBAOS_GREETING=0, and every shell started below it skips. Off for
      # good with `dot-greeting off`, which writes
      # ~/.config/kubaos/terminal/zsh/no-greeting, or with KUBAOS_GREETING=0 in the
      # environment or the user layer's zshrc, which runs just before.
      (lib.mkOrder 1950 ''
        if [[ $TERM_PROGRAM == ghostty && -z $ZSH_EXECUTION_STRING && -t 1
          && -z $TMUX$NVIM$SSH_CONNECTION$SSH_TTY && $KUBAOS_GREETING != 0
          && ! -e $_dot_user/${apps.zsh}/no-greeting ]]; then
          export KUBAOS_GREETING=0
          ${lib.getExe pkgs.fastfetch} --config "${config.xdg.configHome}/fastfetch/greeting.jsonc"
        fi
      '')

      # Must be the very last thing in ~/.zshrc.
      (lib.mkOrder 2000 (
        cachedInit "zoxide" (
          [
            (lib.getExe config.programs.zoxide.package)
            "init"
            "zsh"
          ]
          ++ config.programs.zoxide.options
        )
      ))
    ];

    # Autologin on tty1 starts Hyprland. The snippet is owned by system/
    # and must be the last thing in ~/.zprofile.
    profileExtra = lib.mkOrder 2000 ''
      source ${../system/files/hyprland-autostart.zsh}
    '';
  };

  # Colors: FZF_DEFAULT_OPTS_FILE (generated, see initContent above).
  programs.fzf =
    let
      # Hidden files, but not the noise: VCS dirs, Rust and JS build output.
      fd = "${lib.getExe pkgs.fd} --hidden --follow --exclude .git --exclude target --exclude node_modules";
    in
    {
      enable = true;
      # Initialised from the cache in initContent, where the module would.
      enableZshIntegration = false;
      defaultCommand = "${fd} --type f";
      defaultOptions = [
        "--height=60%"
        "--layout=reverse"
        "--border=rounded"
        "--info=inline-right"
        "--prompt='❯ '"
        "--pointer=▌"
        "--marker=▌"
      ];
      fileWidgetCommand = "${fd} --type f";
      fileWidgetOptions = [
        "--preview '${lib.getExe pkgs.bat} --color=always --style=numbers,changes --line-range=:300 {}'"
        "--preview-window 'right,60%,border-left'"
      ];
      changeDirWidgetCommand = "${fd} --type d";
      changeDirWidgetOptions = [
        "--preview 'LS_COLORS= ${lib.getExe pkgs.eza} --tree --level=2 --icons --color=always {}'"
      ];
    };

  programs.atuin = {
    enable = true;
    # Initialised from the cache in initContent, with these flags.
    enableZshIntegration = false;
    flags = [ "--disable-up-arrow" ];
    # Colors from the wallpaper theme: themes/matugen.toml below.
    settings.theme.name = "matugen";
  };
  # Generated by dot-theme; a dangling link until the first run, and atuin
  # falls back to its plain theme meanwhile.
  xdg.configFile."atuin/themes/matugen.toml".source =
    config.lib.file.mkOutOfStoreSymlink "${config.home.homeDirectory}/.local/state/theme/atuin.toml";

  programs.direnv = {
    enable = true;
    # Initialised from the cache in initContent.
    enableZshIntegration = false;
    nix-direnv.enable = true;
  };

  programs.zoxide = {
    enable = true;
    # Initialised from the cache at the end of initContent (the module's own
    # hook runs at order 851, before fzf/atuin/oh-my-posh).
    enableZshIntegration = false;
    options = [
      "--cmd"
      "cd"
    ];
  };
}
