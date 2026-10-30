# Building a UI

How anything a person sees gets built in KubaOS: a new app, a new shell surface, or a new part of one. Every session and every agent follows it, whoever asked and however small the piece looks. Read it before you plan, brief or write any UI.

The aim is that building an app is assembly: the parts are designed, built and approved before the app exists, so the app's own work is wiring them together and making them do the job. A UI that is designed while it is being built takes many rounds of feedback; one assembled from approved parts takes one.

Everything a person sees goes through three stages, in order: a **sketch** (imperfect, it shows an idea), a **mockup** (perfect, it shows the actual thing) and the **build** (the thing in its toolkit, matching its mockup). **The mockup is the single source of truth** for every UI, in every toolkit: when a build and its mockup disagree, the mockup is right, unless Kuba changes the mockup first. The mockup skill (`.claude/skills/mockup/SKILL.md`) is how a sketch and a mockup are done and what a mockup page must show; this file places those stages among the rest.

## The levels

`docs/DESIGN.md`, "Levels and where they live", defines them: tokens, atoms, molecules, organisms, templates, apps. In short:

- **Atoms, molecules and templates live in the component library**, in both toolkits (`kubaos-ui` and `KubaOS.Components`), each with its page under `docs/design/components/` and its section in the specimen; a part only the shell has may be in `KubaOS.Components` alone, with its section on the QML specimen's "Shell only" page, which the parity check skips (`docs/DESIGN.md`, "Levels and where they live").
- **Organisms live in the app.** They are built only from library parts.
- **An app-only molecule** may live in its app when it means something only there and is unlikely to be needed elsewhere. It is still built only from library atoms. The moment a second surface needs it, it moves into the library (moved, never copied) and goes through the same review as any new component.

An app never draws its own button, row, list, header, sidebar, divider or selected fill: those are the library's. If the library has no part that fits, the library is missing something, and that is found and fixed before the app is built (below), never worked round in the app.

## What a part covers

A component or template is designed and built whole, so nobody decides its details later while building an app. Its mockup page shows everything the mockup skill lists (variants and sizes, every state, content extremes, motion, the keyboard, measures and reflow, light and dark at every scale), each line shown or marked as not applying, with why; its component page under `docs/design/components/` and its specimen sections follow the mockup.

The keyboard covers where focus goes on Tab and Shift+Tab, what the arrows, Enter, Space and Esc do, and where focus lands when the part appears or a page changes (`docs/DESIGN.md`, "The keyboard" and "Focus"). The reflow covers, for a template, its minimum size up to a full screen, and for a window, its default size and its minimum (an app is a normal window: `docs/agents/design-review.md`, "An app is a window").

## The order

### 1. Inventory

Before anything is sketched or built, list every part the new UI needs, screen by screen, against the library (`docs/DESIGN.md`, "The components", the specimen and the mockups). Each part is one of:

- **In the library as it is.**
- **In the library, but needs more:** a variant, a size, a state or a behaviour it doesn't have yet.
- **New**, saying its level (atom, molecule, template) and, for a molecule that would stay in the app, why it is app-only.

Kuba reviews the inventory. Nothing is sketched until he agrees what is new and what changes.

### 2. Sketch with Kuba

Every new part and every change to an existing one is sketched with Kuba, as the mockup skill says, until he explicitly approves the idea. Then the app's own screens are sketched as the approved parts put together, so the layout is agreed before anything is mocked up.

### 3. Mock it up

Each approved sketch becomes its mockup page in the mockup site (`design/mockups/`, `just mockups`; `design/mockups/README.md` says how a page is added), complete and exact, and Kuba approves it (`docs/agents/design-pr-loop.md`, "Reviewing a mockup"). A part that exists already is mocked up as its current approved build is, pixel for pixel, and every difference goes to Kuba as a question. Nothing is built before its mockup is approved.

### 4. Build the library first

Each approved part is built into the library on its own: one piece of agent work per part (or per small group of related parts), in both toolkits where both use it, with its page under `docs/design/components/` and its section in the specimen, matching its mockup. `just mockups-check` holds its boxes to the mockup's, and its shots are compared with the mockup's at every scale, in both modes. Kuba reviews each part himself, live in the specimen (the Command menu's "Design specimen" Item), not through a check or an agent's report. Feedback changes the mockup first, then the library, until he approves it.

### 5. Assemble the app

Only when every part is in the library and approved does an agent build the app. Its brief names the approved mockup pages and parts, and the issue comments holding Kuba's decisions. It assembles the screens from library parts, builds the organisms and the app-only molecules the inventory agreed, and implements what the app does: its data, its backend, its Command menu Items. It creates no other visual building block, and its screens match their mockups.

If the assembling agent finds a gap (a part that doesn't exist, a state its mockup lacks, or a part that can't do what a screen needs), it stops and reports it. The gap goes back through the sketch and the mockup; it is never patched into the app.

### 6. Try it

Kuba tries the app live (`docs/agents/design-pr-loop.md`, "Live preview"). Feedback about how something looks or behaves changes the mockup first, then the part it belongs to: a library part in the library (and seen in the specimen), an organism in the app.

## Which toolkit

`docs/DESIGN.md`, "Levels and where they live", says which surfaces use which library: an app (a window you work in) is built with `kubaos-ui` in Rust and iced, the shell's surfaces with `KubaOS.Components` in QML. Decide it in the inventory, before anything is sketched.
