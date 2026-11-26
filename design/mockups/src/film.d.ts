// The media player's film, as the dev server finds it (`vite.config.ts`, `filmPlugin`).

declare module "virtual:kubaos-film" {
  /** The manifest `just mockup-film` writes, its files served under `base`; null where it isn't fetched. */
  const film: {
    readonly base: string;
    readonly title: string;
    readonly by: string;
    /** In seconds, as the cut's file reads. */
    readonly length: number;
    readonly video: string;
    /** Each one's start in seconds. */
    readonly chapters: readonly { readonly at: number; readonly name: string }[];
    readonly audio: readonly { readonly name: string }[];
    readonly subtitles: readonly { readonly lang: string; readonly name: string; readonly file: string }[];
  } | null;
  export default film;
}
