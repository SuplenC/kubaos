# `kubaos-depth` (home/curated.nix): a wallpaper's foreground cut-out and the
# lock clock's place and tint on it, which the wallpaper-depth Suite runs when
# a wallpaper is set (home/suites/wallpaper-depth.nix).
#
# ONNX Runtime is nixpkgs' (1.24, the API `ort`'s `api-24` feature asks for),
# loaded when the models are: its path is baked in through KUBAOS_DEPTH_ORT
# (src/models.rs), which also keeps it in the package's closure. Nothing is
# downloaded at build time.
{
  lib,
  buildCrate,
  onnxruntime,
}:

buildCrate "kubaos-depth" {
  pname = "kubaos-depth";

  KUBAOS_DEPTH_ORT = "${lib.getLib onnxruntime}/lib/libonnxruntime.so";

  meta = {
    description = "KubaOS's wallpaper depth: a picture's foreground cut-out, and the lock clock's place and tint on it";
    license = with lib.licenses; [
      mit
      asl20
    ];
    mainProgram = "kubaos-depth";
    platforms = lib.platforms.linux;
  };
}
