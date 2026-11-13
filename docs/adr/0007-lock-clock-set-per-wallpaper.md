# 7. The lock's clock: one default, set per wallpaper by the person

Status: accepted (2026-11-13). Supersedes the part of ADR 0004 that has the wallpaper tool choose and ship the clock's place for each picture.

## Context

ADR 0004 has the depth tool place the lock's rest clock on each picture and ship that place, a point in shares of the picture, beside the cut-out. The lock then reads it.

That only works for pictures that went through the tool and were checked by eye. A picture the person adds has no shipped place, so the lock has to fall back to a place of its own anyway. And a place a tool picked is a guess at taste: where the clock reads best, how big it is and whether it stands behind the city are things a person has an opinion on, picture by picture.

## Decision

**The clock has one default place, the same on every wallpaper**: centred, its baseline a fixed share of the screen's height down, one share for a wide screen and one for a portrait screen. No place ships with a picture and no tool chooses one.

**The person sets the clock per wallpaper, in the wallpaper picker.** A picture's settings are its own; a picture never set shows the default. For each picture, and for a wide and a portrait screen apart, the person can set:

- the clock's place, by dragging it, kept inside a margin from the screen's edges;
- its size, between a least and a most share of the screen's shorter side;
- its depth, where the picture has depth: a slider of steps, one per layer of the picture, the lowest step the furthest behind and the highest in front of everything. A picture with one layer has one step; a picture with none has no slider;
- its colour, picked for it from the picture's scheme or set by hand;
- a glow under it for contrast: off, dark, light or picked for it, and its strength;
- the date: its size, its alignment to the digits and its gap from them. The date stands at the clock's depth;
- whether the lock shows the clock and the date at all. The date is not shown without the clock.

The shipped depth files keep the cut-out, the plate and the tint. The tint is what the colour is when it is picked for the person.

## Consequences

- The tool no longer places the clock, and no place ships beside a picture.
- The depth slider needs one traced mask per layer. `kubaos-depth` writes one cut-out today, so every shipped picture has at most one step until it writes a mask per layer.
- The settings are kept per wallpaper and per screen shape.
- With the clock off there is nothing to fly between the lock and the bar, so the lock and unlock motion has no clock flight for that picture.
- A person's picture gets the same editor as a shipped one. Without a cut-out it has no depth, and everything else works.
