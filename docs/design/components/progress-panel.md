# ProgressPanel

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

## `ProgressPanel`

QML `ProgressPanel`; *QML only: its `kubaos-ui` twin comes with the list and progress molecules (#304). Until then it is in neither specimen, so the parity check leaves it out.*

A job followed to its end, as Kōbō's install screen shows one: the Command menu's progress View. A column as wide as the content column (`column`) at most, centred in the panel's room, its parts 16 apart and centred: the job's state as the panel's one big symbol (48), its title in `headline`, a `ProgressBar` while it knows how far it is, and a `body` line with its state and what it last said ("42% · Downloading", "Done", "Failed · …").

`job` is `{title, state, progress, message}`: `state` `running`, `succeeded` or `failed`, `progress` from 0 to 1, or below 0 while it can't tell. While it can't tell, the symbol is a `Spinner`; once it can, an hourglass in `primary` over the bar, a check in `primary` with the bar full when it succeeds, and an error symbol in `error` with the line in `error` and no bar when it fails. The bar follows the job as every `ProgressBar` does. The spinner shows only while the job can't tell how far it is, since a running animation makes every frame wait on the screen's refresh, and with `moves` off (the menu's motion set to fades) it stands still.
