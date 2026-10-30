# Chip

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `Chip`

Rust `chip::chip`; QML `Chip`.

A short fact or pick, 24 tall (`chip`) at the control radius, 12, which on a chip is a pill, `label` type, an optional 16 icon 4 from the label, an optional close. A pick may also be pressed, with the state layer: Kōbō's picked layouts, where a press makes one the default, and a setting a toolbar turns on and off (the keyboard section's Mac accents, `Selected` while on, `Neutral` while off), which in the shell takes the keyboard too and shows the focus tube. In QML a pressable chip may be no stop for Tab (`tabStop: false`) where the field or the row beside the chips is the stop and a click picks one: the clock's timer presets and settings. A pressable chip's `tooltip` (QML too) names it in a `Hint` under it while the pointer or the keyboard is on it. *iced's `chip::chip` has neither yet.* Its left side pads 12, or 8 with an icon; its right pads 12, or nothing when the close button (a 24 circle) ends it. Kinds: `Neutral` (a fact), `Selected` (one of several picks), `Accent` (a tag that matters: Recommended, Default), `Cold` (rarely: demo, preview), `Filled` (the accent's own fill: an action among states, Reboot beside Done and To do).

**States:** a plain chip has none. A pressable one is resting, hovered, pressed, focused and disabled (`Chip::disabled`, `enabled: false`): its content at the disabled opacity and the chip at its own, no press, and no stop for Tab; its close button is resting, hovered and pressed, and disabled with it.
