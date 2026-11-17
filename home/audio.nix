# Audio's switches (scripts/dot-audio): noise suppression and echo
# cancellation as PipeWire filters, each a user unit running a PipeWire
# process of its own, and "New devices take over" as a WirePlumber script.
# Ubuntu's PipeWire and WirePlumber are the session's (system/packages.toml,
# group audio); this adds to them without restarting either.
#
# Each switch is a flag file in the user layer's audio folder, so a login
# brings it back as it was left: a filter's unit is wanted by PipeWire and
# starts only while its flag exists, and the takeover's unit copies its flag
# into WirePlumber's `kubaos.takeover` setting each time WirePlumber starts.
# WirePlumber loads the script from
# config/wireplumber/wireplumber.conf.d/50-kubaos.conf at its start, which the
# defaults link like any other config file.
{
  config,
  kubaosPkgs,
  lib,
  pkgs,
  ...
}:
let
  userLayer = import ./user-layer.nix { configHome = config.xdg.configHome; };
  # scripts/dot-audio owns the flag files.
  flag = name: "${userLayer.dir "audio"}/${name}";

  # A filter's unit: KubaOS's PipeWire, as a client of the session's, on a
  # config of its own. It stops with PipeWire and comes back with it.
  filterUnit =
    {
      description,
      flagName,
      conf,
    }:
    {
      Unit = {
        Description = description;
        ConditionPathExists = flag flagName;
        After = [
          "pipewire.service"
          "wireplumber.service"
        ];
        BindsTo = [ "pipewire.service" ];
      };
      Service = {
        ExecStart = "${lib.getExe' kubaosPkgs.pipewire "pipewire"} -c ${conf}";
        Restart = "on-failure";
        RestartSec = 2;
      };
      Install.WantedBy = [ "pipewire.service" ];
    };
in
{
  systemd.user.services = {
    kubaos-noise-suppression = filterUnit {
      description = "Noise suppression for the microphone (RNNoise)";
      flagName = "noise-suppression";
      conf = pkgs.replaceVars ./files/noise-suppression.conf { rnnoise = kubaosPkgs.rnnoise-plugin; };
    };
    kubaos-echo-cancellation = filterUnit {
      description = "Echo cancellation for the microphone";
      flagName = "echo-cancellation";
      conf = ./files/echo-cancellation.conf;
    };
    kubaos-audio-takeover = {
      Unit = {
        Description = "New devices take over: the setting into WirePlumber";
        After = [ "wireplumber.service" ];
        PartOf = [ "wireplumber.service" ];
      };
      Service = {
        Type = "oneshot";
        ExecStart = "${pkgs.bash}/bin/bash ${../scripts/dot-audio} takeover sync";
      };
      Install.WantedBy = [ "wireplumber.service" ];
    };
  };

  # WirePlumber looks for scripts under ~/.local/share/wireplumber/scripts.
  xdg.dataFile."wireplumber/scripts/kubaos/takeover.lua".source = ./files/kubaos-takeover.lua;
}
