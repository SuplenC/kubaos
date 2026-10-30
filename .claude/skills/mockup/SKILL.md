---
name: mockup
description: Sketch, then mock up, any KubaOS UI before it is built. Use when a new or changed screen, surface, component, template, state or animation comes up, when Kuba raises a visual point, or when a piece of mockup work (#307) is briefed or done.
---

# Sketch, then mockup, then build

Everything a person sees in KubaOS goes through three stages, in order:

1. **Sketch.** Imperfect. It shows an idea.
2. **Mockup.** Perfect. It shows the actual thing.
3. **Build.** The thing in its toolkit, matching its mockup.

The **mockup is the single source of truth** for every UI, whatever the toolkit (iced, QML, anything later) and wherever it runs. Kuba approves each stage before the next starts. Liking a draft, or adding a tweak to it, is not approval; approval is him saying so.

`docs/agents/building-ui.md` places these stages among the rest (the inventory before, the library and the app after). This skill is how the sketch and the mockup are done.

## 1. Sketch

A sketch answers "is this the right idea?": the layout, the hierarchy, the behaviour, the motion, at a glance. It may cut corners. Its icons, copy and data may be stand-ins.

- Draw it as one self-contained HTML page with the real token values (`design/tokens.toml`), and render it in the conversation (`html_preview` to check, then `html_render`).
- Make it **live** wherever behaviour is the question: drag, hover, scroll, resize, keys, a width slider. A still image of an interaction settles nothing.
- Keep the files in `~/.cache/kubaos-sketches/<issue>/`, where agents can read them; `/tmp` is RAM.
- Put each open choice to Kuba as a numbered decision, with your recommendation.
- After each round, record what Kuba decided on the issue as a comment that says how the thing is now, with every measure, token and key.

**Done when** Kuba has approved the idea explicitly, and every decision is recorded on the issue.

## 2. Mockup

A mockup is the thing itself, complete, in the mockup site: `design/mockups/`, served with hot reload by `just mockups`. Its values come from `design/tokens.toml` alone, so a mockup can't leave the scale. Kuba can open it, touch it and adjust it himself.

`design/mockups/README.md` says how a page is added: its required parts, and where it is listed.

A mockup page is **exhaustive**. It shows and describes every one of these that applies, live, next to its written description:

- **Variants and sizes,** and which is for what.
- **Every state:** rest, lit (hovered or focused from the keyboard, one state), pressed, disabled, selected, loading, empty, error, by night and by day (`docs/DESIGN.md`, "States").
- **The lit line:** each tube, core or knob a lit control burns at night, its glow, and its line by day, when nothing glows.
- **Content extremes:** the shortest and longest text, a translation 40% longer, a missing icon or picture, none, one and many.
- **Motion:** how it arrives, leaves and moves between states, each carrying its reproducible recipe as data: every part that moves, each property's from and to by token, each part's duration, cue and curve by token, a stepped move's keyframes, and what reduced motion and the speed make of it. The player animates from that same data, looping, with pause, restart, slow motion, a plot of every track's curve over the loop to seek on, and each part of a move of several played alone beside the whole, so a library builds the same move from the page alone.
- **The keyboard:** every key, where focus goes, and where it lands when the part appears or a page changes.
- **Measures:** every box, padding and gap by its token, and the reflow from the minimum size to a full screen, across window sizes.
- **Light and dark, at scales 1, 1.25, 1.5 and 2.**

"Applies" is the only way out of a line. Tedious detail is the work: a part described loosely becomes several implementations that drift apart.

**For a part that already exists,** the truth to match is its current approved build: both specimens and the surfaces' shots (`just shots`). The mockup matches it pixel for pixel. Every difference goes to Kuba as a question, with both pictures; a mockup never settles a difference silently.

**For a new part,** the approved sketch and Kuba's recorded decisions are the input. Every gap the sketch left open (a state, a size, a key, a motion) is filled in the mockup and asked about.

Every mocked element carries the specimens' probe, so the geometry check can hold the builds to it.

**Done when** every line above is shown or marked as not applying, every difference from the current build is asked about, and Kuba has approved the mockup page.

## 3. Hand over to the build

The build brief names the approved mockup pages by path, and the issue comments holding Kuba's decisions. It says the build must match the mockup, and that a gap stops the build: a part or a state the mockup lacks goes back through sketch and mockup, never into the app. The build's review compares its shots with the mockup's at every scale, in both modes.

When the mockup and a build disagree, the mockup is right, unless Kuba changes the mockup first.
