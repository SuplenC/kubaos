# zsh, ported from the macOS ~/.zshrc and ~/.zprofile.
#
# programs.zsh writes ~/.zshrc, ~/.zshenv and ~/.zprofile. Order of
# programs.zsh.initContent (lower runs first):
#   800   oh-my-zsh (plugins: git, no theme)
#   900   FZF_CTRL_R_COMMAND="" (keeps fzf off Ctrl-R)
#   ~910  fzf (Ctrl-T, Alt-C)
#   1000  direnv hook, atuin (Ctrl-R), aliases, oh-my-posh
#   1900  the user layer's zshrc (~/.config/kubaos/zsh/zshrc)
#   2000  zoxide as `cd`, last, so `zoxide doctor` stays quiet
#
# PATH for proto/bun/cargo/~/.local/bin lives in home.sessionPath (nix agent).
{
  config,
  pkgs,
  lib,
  ...
}:
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
    };

    shellAliases = {
      zshrc = "nvim ~/.config/kubaos/zsh/zshrc";
      wipe = "clear && tmux clear-history";
      nvimconfig = "nvim ~/.config/nvim";
      tma = "tmux attach";
      arms-dump = "sh ~/Scripts/arms_dump.sh";
    };

    initContent = lib.mkMerge [
      # Keeps fzf off Ctrl-R (atuin owns it). home-manager 26.05 has no option
      # for this; fzf's key-bindings.zsh skips Ctrl-R when it is set but empty.
      (lib.mkOrder 900 ''
        export FZF_CTRL_R_COMMAND=""
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
          [[ -r $_dot_user/lazygit/config.yml ]] && lg+=("$_dot_user/lazygit/config.yml")
          (( $#lg > 1 )) && export LG_CONFIG_FILE="''${(j:,:)lg}"
        }
        [[ -r $_dot_user/bacon/prefs.toml ]] && export BACON_PREFS="$_dot_user/bacon/prefs.toml"
        # oh-my-zsh exports LS_COLORS (dircolors) and eza lets it override
        # ~/.config/eza/theme.yml; keep LS_COLORS for ls, fd and completion.
        function eza() { LS_COLORS= command eza "$@"; }
        function glow() {
          local style="$HOME/.local/state/theme/glow.json"
          if [[ -r $style ]]; then command glow -s "$style" "$@"; else command glow "$@"; fi
        }

        # oh-my-posh: themed config from dot-theme if present, else the static
        # one. `dot-theme set` rewrites ohmyposh.toml; _dot_omp_refresh runs
        # before each prompt and re-inits oh-my-posh when the file's mtime
        # changed, so open shells pick up the new colors at their next prompt
        # (one stat per prompt, no fork).
        zmodload -F zsh/stat b:zstat
        typeset -g _dot_omp_mtime=""
        function _dot_omp_init() {
          local cfg="$_dot_theme/ohmyposh.toml"
          local -a st
          [[ -r $cfg ]] || cfg="$HOME/.config/ohmyposh/config.toml"
          zstat -A st +mtime -- "$cfg" 2>/dev/null || st=(0)
          [[ "$cfg:$st[1]" == "$_dot_omp_mtime" ]] && return
          _dot_omp_mtime="$cfg:$st[1]"
          eval "$(${lib.getExe pkgs.oh-my-posh} init zsh --config "$cfg")"
        }
        _dot_omp_init
        precmd_functions=(_dot_omp_init ''${precmd_functions:#_dot_omp_init})
      ''

      # The user layer's zshrc, after everything but zoxide, so it wins.
      (lib.mkOrder 1900 ''
        [[ -r $_dot_user/zsh/zshrc ]] && source "$_dot_user/zsh/zshrc"
      '')

      # Must be the very last thing in ~/.zshrc.
      (lib.mkOrder 2000 ''
        eval "$(${lib.getExe config.programs.zoxide.package} init zsh ${lib.escapeShellArgs config.programs.zoxide.options})"
      '')
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
      enableZshIntegration = true;
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
    enableZshIntegration = true;
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
    enableZshIntegration = true;
    nix-direnv.enable = true;
  };

  programs.zoxide = {
    enable = true;
    # Initialised by hand at the end of initContent (the module's own hook
    # runs at order 851, before fzf/atuin/oh-my-posh).
    enableZshIntegration = false;
    options = [
      "--cmd"
      "cd"
    ];
  };
}
