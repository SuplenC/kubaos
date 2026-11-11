//! KubaOS's wallpaper depth: a picture's foreground cut out, which the lock draws over its rest
//! clock so the city stands in front of the digits, and the clock's place and tint on it.
//!
//! ```text
//! picture ─┬─▶ Depth Anything V2 Small ─▶ the sky's candidate ────┐
//!          └─▶ BiRefNet lite ───────────▶ the object's candidate ─┴─▶ scored under the clock
//!                                                                         │ one kept, or none
//!                          the sky's edge: SAM, then keyed at full size ◀─┤
//!                                         the object's: BiRefNet's own ◀──┘
//!          cutout.png + depth.toml ◀── the clock's place, its tint
//! ```
//!
//! # One cut-out, or none
//!
//! Each candidate is a soft mask of what stands in front of the clock: for the sky, everything
//! nearer than the depth map's floor, where Depth Anything puts the sky; for the object,
//! BiRefNet's chances. A bad cut-out is worse than none, so one is kept only when all of this
//! holds:
//!
//! 1. **It is a candidate at all.** The sky holds between a tenth and four fifths of the picture
//!    and, joined to the top edge, at least half its top row; the object between half a percent and
//!    three fifths of it.
//! 2. **Its two sides separate**, by at least [`choose::SEPARATION_MIN`], 0.85. For the sky, 1 less
//!    the pixels just past its depth (haze, a far mountain) for each of its own; for the object,
//!    Otsu's between-class variance of BiRefNet's chances as a share of their whole, 1 for an
//!    object cut clean and 0.75 for chances spread evenly.
//! 3. **The clock has a place on it** (`clock`): somewhere the cut-out covers between
//!    [`clock::COVER_MIN`] and [`clock::COVER_MAX`] of the digits on the Lock page's screen, and
//!    none of the kicker (the date line) and its halo on any of [`clock::SCREENS`].
//! 4. **It is sure of itself under the clock.** Its score, the separation times the mean of `|2p −
//!    1|` over the digits' box at that place, reaches [`choose::SCORE_MIN`], 0.7.
//!
//! Of the candidates that pass, the higher score is kept. A sky's edge is then traced by SAM and
//! keyed out at the picture's size (`edge`), and the clock is placed again on the traced mask; an
//! object keeps BiRefNet's edge. With none kept, no cut-out is written and the
//! clock takes [`clock::DEFAULT_PLACE`]. The tint is the picture's under the digits either way
//! (`tint`).
//!
//! # The models
//!
//! Never in this repository: the wallpaper-depth Suite pins and fetches them, and the folder
//! given to `--models` holds them under these names ([`models`]).
//!
//! | File | Model | Licence | Source |
//! |---|---|---|---|
//! | `depth-anything-v2-small.onnx` | Depth Anything V2 Small | Apache-2.0 | <https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/main/onnx/model.onnx> |
//! | `birefnet-lite.onnx` | BiRefNet, general, Swin-T backbone | MIT | <https://github.com/danielgatis/rembg/releases/download/v0.0.0/BiRefNet-general-bb_swin_v1_tiny-epoch_232.onnx> |
//! | `sam-vit-b-encoder.onnx` | Segment Anything ViT-B, image encoder | Apache-2.0 | <https://github.com/danielgatis/rembg/releases/download/v0.0.0/sam_vit_b_01ec64.encoder.onnx> |
//! | `sam-vit-b-decoder.onnx` | Segment Anything ViT-B, mask decoder | Apache-2.0 | <https://github.com/danielgatis/rembg/releases/download/v0.0.0/sam_vit_b_01ec64.decoder.onnx> |
//!
//! Depth Anything V2's larger sizes are licensed for non-commercial use only; Small is the one
//! that isn't.
//!
//! # Owns
//!
//! - The run ([`run`]), the candidates and the choice ([`choose`]), SAM's edge ([`edge`]), the
//!   clock's geometry ([`clock`]) and tint ([`tint`]), the models ([`models`]) and the files
//!   written ([`sidecar`]).
//!
//! # Not here
//!
//! - Fetching the models, the hook that runs this when a wallpaper is set, and its systemd job: the
//!   wallpaper-depth Suite. Drawing the cut-out and the clock: the lock,
//!   `config/quickshell/modules/lock/`.

pub mod choose;
pub mod clock;
pub mod edge;
pub mod grid;
pub mod models;
pub mod run;
pub mod sidecar;
pub mod tint;

pub use self::run::run;
pub use self::sidecar::Sidecar;
