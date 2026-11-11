# mise, the Suites' toolchain manager, at the version this release pins.
# Only a KubaOS release moves it: `kubaos` reads mise's JSON and passes its
# flags, so the two are built and rolled back together. nixpkgs' own mise
# trails upstream by months and keeps `self_update`, which would let mise
# replace itself under the release.
#
# Built without the `self_update` feature, so `mise self-update` refuses
# whatever its flags (docs/packaging.md in mise's source), with the
# instructions file mise prints when it does, or when it finds a newer
# release. nixpkgs' derivation gives the build inputs; its patches are for
# its own version and don't apply here.
{
  lib,
  mise,
  fetchFromGitHub,
  rustPlatform,
  git,
  cmake,
  writeText,
}:
let
  updateInstructions = writeText "mise-self-update-instructions.toml" ''
    message = "mise comes with KubaOS and moves with its releases: run `kubaos update`.\n"
  '';
in
mise.overrideAttrs (
  finalAttrs: old: {
    version = "2026.11.6";
    src = fetchFromGitHub {
      owner = "jdx";
      repo = "mise";
      tag = "v${finalAttrs.version}";
      hash = "sha256-3F3KGCq2HeNRJVGPG9juojmhrPObZM6l0r6lIa54hXg=";
    };
    cargoDeps = rustPlatform.fetchCargoVendor {
      inherit (finalAttrs) pname version src;
      hash = "sha256-nN/TdmKrNtQIyVhRzKlYPBEh8yGB1+CZli2fjlO/OTo=";
    };

    # libz-ng-sys, new in this version's dependencies, builds zlib-ng with cmake.
    nativeBuildInputs = old.nativeBuildInputs ++ [ cmake ];
    dontUseCmakeConfigure = true;

    buildNoDefaultFeatures = true;
    buildFeatures = [
      "native-tls"
      "vfox/vendored-lua"
    ];
    # mise's own tests take most of an hour and reach the network.
    doCheck = false;

    # mise runs git for its plugins; a home's PATH may have none.
    postPatch = ''
      substituteInPlace ./src/git.rs --replace-fail 'Command::new("git")' 'Command::new("${lib.getExe git}")'
    '';

    postInstall = ''
      installManPage ./man/man1/mise.1
      installShellCompletion \
        --bash ./completions/mise.bash \
        --fish ./completions/mise.fish \
        --zsh ./completions/_mise
      install -Dm444 ${updateInstructions} $out/lib/mise/mise-self-update-instructions.toml
    '';
  }
)
