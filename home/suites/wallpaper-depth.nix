# The wallpaper-depth Suite: kubaos-depth (crates/kubaos-depth), the four
# models it runs, and kubaos-depth-queue (wallpaper-depth-queue.sh), which
# `scripts/dot-theme` finds on PATH and calls when a still of the user's own is
# set. Nothing until kubaos.d/suites/wallpaper-depth.toml exists; the metadata
# is the `wallpaper-depth` entry in crates/kubaos-core/src/suites/catalog.rs.
#
# The models are fetched by Nix, pinned by hash, and joined into one folder
# under the names `--models` expects (the crate's doc lists them), about 700 MB
# in the store. Removing the Suite takes them, the tool and the command out of
# the next generation; the results, under ~/.cache/kubaos/wallpapers, are the
# Suite's cache.
{
  config,
  lib,
  pkgs,
  kubaosPkgs,
  ...
}:
let
  suite = config.kubaos.suites.wallpaper-depth or null;
  tool = kubaosPkgs.kubaos-depth;

  models = pkgs.linkFarm "kubaos-depth-models" {
    # Apache-2.0. Its Base and Large sizes are non-commercial, so only Small.
    "depth-anything-v2-small.onnx" = pkgs.fetchurl {
      url = "https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/main/onnx/model.onnx";
      hash = "sha256-r7alwo87a/FhjG5D8CBz7539xw6TdQLVFgPlewod8Qw=";
    };
    # MIT. BiRefNet, general, with the Swin-T backbone.
    "birefnet-lite.onnx" = pkgs.fetchurl {
      url = "https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx";
      hash = "sha256-VgACQ3b1cqVXhwpesK+x5ZYWNr704eIhMgJUZ9DwMzM=";
    };
    # Apache-2.0. Segment Anything ViT-B's image encoder.
    "sam-vit-b-encoder.onnx" = pkgs.fetchurl {
      url = "https://github.com/danielgatis/rembg/releases/download/v0.0.0/sam_vit_b_01ec64.encoder.onnx";
      hash = "sha256-YqKcOevIrk38PZ7I6uKMKO/INgIz+o/7oinrn2rAwwo=";
    };
    # Apache-2.0. Segment Anything ViT-B's mask decoder.
    "sam-vit-b-decoder.onnx" = pkgs.fetchurl {
      url = "https://github.com/danielgatis/rembg/releases/download/v0.0.0/sam_vit_b_01ec64.decoder.onnx";
      hash = "sha256-mCCkii0AKurXtNqyBNgTF2qBCl8dyPMuMvdZzAcK6X4=";
    };
  };

  # systemd-run, systemctl and systemd-ac-power stay the host's: the user
  # manager they talk to is Ubuntu's.
  queue = pkgs.writeShellApplication {
    name = "kubaos-depth-queue";
    runtimeInputs = [ pkgs.coreutils ];
    text = ''
      export KUBAOS_DEPTH_TOOL=${lib.getExe tool}
      export KUBAOS_DEPTH_VERSION=${lib.escapeShellArg tool.version}
      export KUBAOS_DEPTH_MODELS=${models}
    ''
    + builtins.readFile ./wallpaper-depth-queue.sh;
  };
in
{
  config = lib.mkIf (suite != null) {
    home.packages = [
      tool
      queue
    ];
  };
}
