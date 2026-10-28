//! The neon sign's tokens for Kōbō's done screen, `crates/kubaos-kobo/src/done/neon.rs`: how a
//! tube strikes and the クバ mark's strokes, the same the boot splash strikes its sign with and the
//! logo's SVGs draw.
//!
//! # Owns
//!
//! - The module's text.
//!
//! # Not here
//!
//! - Drawing the mark: `crates/kubaos-kobo/src/done/mark.rs`. The splash's copy: [`crate::boot`].

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::model::Tokens;
use crate::model::Tone;
use crate::target::Kept;
use crate::target::OUT_DIR_HEADER;
use crate::target::Target;

/// `crates/kubaos-kobo/src/done/neon.rs`.
#[derive(Debug)]
pub struct KoboNeon;

impl Target for KoboNeon {
    fn path(&self) -> &'static str {
        "crates/kubaos-kobo/src/done/neon.rs"
    }

    fn kept(&self) -> Kept {
        Kept::OutDir
    }

    fn render(&self, tokens: &Tokens, _root: &Path) -> io::Result<String> {
        let neon = &tokens.neon;
        let mark = &neon.mark;
        let floats = |values: &[f32]| values.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>().join(", ");
        let mut out = String::new();
        for line in OUT_DIR_HEADER {
            let _ = writeln!(out, "// {line}");
        }
        out.push_str(
            "\n\
             use super::mark::Tone;\n\
             \n\
             /// A tube striking: its brightness, 0 to 1, through even steps over `medium`.\n",
        );
        let _ = writeln!(
            out,
            "pub const STRIKE: [f32; {}] = [{}];",
            neon.strike.len(),
            floats(&neon.strike)
        );
        out.push_str("\n/// The drawing's side, in its units.\n");
        let _ = writeln!(out, "pub const UNITS: f32 = {:?};", mark.units);
        out.push_str("/// How far inside the drawing the strokes' coordinates start, in its units.\n");
        let _ = writeln!(out, "pub const MARGIN: f32 = {:?};", mark.margin);
        out.push_str("/// A warm tube's width, in the drawing's units.\n");
        let _ = writeln!(out, "pub const WIDTH: f32 = {:?};", mark.width);
        out.push_str("/// A cold tube's width, of a warm one's.\n");
        let _ = writeln!(out, "pub const COLD_WIDTH: f32 = {:?};", mark.cold_width);
        out.push_str("/// A warm tube's white-hot core's width, of a warm tube's.\n");
        let _ = writeln!(out, "pub const CORE: f32 = {:?};", mark.core);
        out.push_str("/// A cold tube's core's width, of a warm tube's.\n");
        let _ = writeln!(out, "pub const COLD_CORE: f32 = {:?};", mark.cold_core);
        out.push_str(
            "\n/// The strokes in the order a hand writes them, as SVG path data of absolute `M`, `L`, `H` and `Q`\n\
             /// commands, each letter starting a word.\n",
        );
        let _ = writeln!(out, "pub const STROKES: [(&str, Tone); {}] = [", mark.strokes.len());
        for stroke in &mark.strokes {
            let tone = match stroke.tone {
                Tone::Warm => "Warm",
                Tone::Cold => "Cold",
            };
            let _ = writeln!(out, "    ({:?}, Tone::{tone}),", stroke.path);
        }
        out.push_str("];\n");
        Ok(out)
    }
}
