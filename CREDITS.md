# Credits

KubaOS credits every external tool, library, asset, font, data source and service it uses: the people who made them deserve it. This file is where those credits live, each with its creators, its licence and a link. Anything external a pull request adds gets its line here in the same pull request.

## What KubaOS is built on

What KubaOS ships, draws on or is designed around.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| atmos-fx 0.4.0 | Rain, snow and hail over the weather's skies in the mockups (`design/mockups/src/lib/weather-sky.ts`), from npm | Carson Ye and contributors | MIT | [github.com/Whynotmetoo/atmos-fx](https://github.com/Whynotmetoo/atmos-fx) |
| Open-Meteo | The forecast source the clock's weather is designed around | Open-Meteo | See its site | [open-meteo.com](https://open-meteo.com) |
| Tabler Icons | The KubaOS Symbols set (`crates/kubaos-ui/assets/symbols/`) and the style its weather symbols are drawn in | Paweł Kuna | MIT ([`TABLER-LICENSE.txt`](crates/kubaos-ui/assets/symbols/TABLER-LICENSE.txt)) | [github.com/tabler/tabler-icons](https://github.com/tabler/tabler-icons) |

Credits kept beside the files they cover:

- [`design/mockups/src/assets/audio/CREDITS.md`](design/mockups/src/assets/audio/CREDITS.md): the media player mockup's songs.
- [`design/mockups/src/assets/lock/CREDITS.md`](design/mockups/src/assets/lock/CREDITS.md): the Lock mockup's wallpapers and the tools that cut their depth.
- The specimen's icon sets' licences: [Lucide](crates/kubaos-specimen/assets/icons/lucide-LICENSE.txt), [Material Symbols](crates/kubaos-specimen/assets/icons/material-LICENSE.txt), [Phosphor](crates/kubaos-specimen/assets/icons/phosphor-LICENSE.txt) and [Tabler](crates/kubaos-specimen/assets/icons/tabler-LICENSE.txt).
- [`LICENSES/`](LICENSES): the licence texts the third-party files under `apps/*/` name in their headers.
- The wallpapers' credits: [SuplenC/kubaos-wallpapers](https://github.com/SuplenC/kubaos-wallpapers).

## What it is made with

The development tools KubaOS is made with.

| Name | What KubaOS uses it for | Creators | Licence | Link |
|---|---|---|---|---|
| Vite 8.3.5 | Serves and builds the mockups (`design/mockups/`) | Evan You, VoidZero Inc. and Vite contributors | MIT | [vite.dev](https://vite.dev) |
| TypeScript 7.0.2 | The mockups' language and their type check | Microsoft Corporation | Apache-2.0 | [github.com/microsoft/TypeScript](https://github.com/microsoft/TypeScript) |
| Playwright 1.59.1 (`playwright-core`) | Drives the headless browser that renders the mockups' shots and checks | Microsoft Corporation | Apache-2.0 | [playwright.dev](https://playwright.dev) |
| `@types/node` 26.6.6 | Node's types for the mockups' scripts | Microsoft Corporation and the DefinitelyTyped contributors | MIT | [github.com/DefinitelyTyped/DefinitelyTyped](https://github.com/DefinitelyTyped/DefinitelyTyped/tree/master/types/node) |
