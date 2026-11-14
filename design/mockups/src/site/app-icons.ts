// The app icons the IconTile's pages measure: one of each kind an icon theme ships, so the measured
// fit shows on a plate, a plate in a wide margin, and shapes round, square and wide.

// KubaOS's own icon, a plate drawn out to the edges of its file with large rounded corners.
import kubaos from "../../../../branding/png/kubaos-icon-128.png?url";
// A vendor's plate, a rounded square, drawn in the middle half of its file with a wide transparent
// margin, as many hicolor icons leave room round theirs.
import plate from "../assets/icon-tile/plate.svg?url";
// A round badge filling its file: a disc covers 78.5% of its square box, so it reads as a shape.
import disc from "../assets/icon-tile/disc.svg?url";
// A glyph with no plate of its own: a star, whose box is square but mostly empty.
import star from "../assets/icon-tile/star.svg?url";
// A wordmark-shaped logo, three times as wide as it is tall: the tile shows above and below it.
import wide from "../assets/icon-tile/wide.svg?url";

export interface AppIcon {
  readonly name: string;
  readonly picture: string;
  readonly for: string;
}

export const APP_ICONS: readonly AppIcon[] = [
  { name: "A full plate", picture: kubaos, for: "fills the tile, cut by its corners" },
  { name: "A plate in a margin", picture: plate, for: "measured, it fills the tile all the same" },
  { name: "A disc", picture: disc, for: "too little of its box: a shape, `space.s4` in" },
  { name: "A glyph", picture: star, for: "square but mostly empty: a shape" },
  { name: "A wide logo", picture: wide, for: "a shape, fitted to the tile's width" },
];
