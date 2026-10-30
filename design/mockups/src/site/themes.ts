// The themes the toolbar can paint every page in, so no page is judged in one accent alone: the
// tokens' own palette, matugen's for the default wallpaper, then three others. Each other is
// matugen's output from a seed colour, as `scripts/dot-theme` makes a wallpaper's, for the roles
// that have a matugen source (`[color.source]`); the fixed roles keep the tokens' values. Made once
// with `matugen color hex SEED -t scheme-fidelity --json hex --dry-run`, in each mode.

export interface Theme {
  readonly slug: string;
  readonly name: string;
  /** The colour matugen was given, or null for the tokens' own palette. */
  readonly seed: string | null;
  /** The matugen roles in each mode, over the tokens' fallback; none for the tokens' own. */
  readonly dark: Readonly<Record<string, string>> | null;
  readonly light: Readonly<Record<string, string>> | null;
}

export const THEMES: readonly Theme[] = [
  { slug: "neon", name: "Neon sign", seed: null, dark: null, light: null },
  {
    slug: "rose",
    name: "Rose",
    seed: "#d4407a",
    dark: { cold: "#6cde74", cold_container: "#30a643", error: "#ffb4ab", error_container: "#93000a", inverse_on_surface: "#3b2d30", inverse_primary: "#b0225f", inverse_surface: "#f4dde1", on_cold: "#00390d", on_cold_container: "#000000", on_error: "#690005", on_error_container: "#ffdad6", on_primary: "#650031", on_primary_container: "#000000", on_secondary: "#591831", on_secondary_container: "#ffdbe3", on_surface: "#f4dde1", on_surface_variant: "#debfc5", on_tertiary: "#00390d", on_tertiary_container: "#000000", outline: "#a68990", outline_variant: "#574146", primary: "#ffb1c6", primary_container: "#f35892", scrim: "#000000", secondary: "#ffb1c6", secondary_container: "#783149", shadow: "#000000", surface: "#1c1013", surface_bright: "#443539", surface_container: "#291c1f", surface_container_high: "#34262a", surface_container_highest: "#403134", surface_container_low: "#25181b", surface_container_lowest: "#160b0e", surface_dim: "#1c1013", surface_variant: "#574146", tertiary: "#6cde74", tertiary_container: "#30a643" },
    light: { cold: "#006b20", cold_container: "#00872b", error: "#ba1a1a", error_container: "#ffdad6", inverse_on_surface: "#ffecef", inverse_primary: "#ffb1c6", inverse_surface: "#3b2d30", on_cold: "#ffffff", on_cold_container: "#ffffff", on_error: "#ffffff", on_error_container: "#410002", on_primary: "#ffffff", on_primary_container: "#ffffff", on_secondary: "#ffffff", on_secondary_container: "#56162f", on_surface: "#25181b", on_surface_variant: "#574146", on_tertiary: "#ffffff", on_tertiary_container: "#ffffff", outline: "#8b7077", outline_variant: "#debfc5", primary: "#ad1f5d", primary_container: "#ce3b76", scrim: "#000000", secondary: "#91465f", secondary_container: "#fea0bc", shadow: "#000000", surface: "#fff8f8", surface_bright: "#fff8f8", surface_container: "#ffe8ec", surface_container_high: "#fae2e7", surface_container_highest: "#f4dde1", surface_container_low: "#fff0f2", surface_container_lowest: "#ffffff", surface_dim: "#ecd4d9", surface_variant: "#fbdae1", tertiary: "#006b20", tertiary_container: "#00872b" },
  },
  {
    slug: "lagoon",
    name: "Lagoon",
    seed: "#1597b8",
    dark: { cold: "#ffb77a", cold_container: "#cc7f34", error: "#ffb4ab", error_container: "#93000a", inverse_on_surface: "#2c3133", inverse_primary: "#00677f", inverse_surface: "#dfe3e6", on_cold: "#4c2700", on_cold_container: "#000000", on_error: "#690005", on_error_container: "#ffdad6", on_primary: "#003543", on_primary_container: "#000000", on_secondary: "#0d3541", on_secondary_container: "#c1e6f5", on_surface: "#dfe3e6", on_surface_variant: "#bdc8ce", on_tertiary: "#4c2700", on_tertiary_container: "#000000", outline: "#879297", outline_variant: "#3e484d", primary: "#6ad4f7", primary_container: "#229dbe", scrim: "#000000", secondary: "#a7ccdb", secondary_container: "#274b58", shadow: "#000000", surface: "#0f1416", surface_bright: "#353a3c", surface_container: "#1b2022", surface_container_high: "#262b2d", surface_container_highest: "#313538", surface_container_low: "#171c1e", surface_container_lowest: "#0a0f11", surface_dim: "#0f1416", surface_variant: "#3e484d", tertiary: "#ffb77a", tertiary_container: "#cc7f34" },
    light: { cold: "#8b4c00", cold_container: "#aa631a", error: "#ba1a1a", error_container: "#ffdad6", inverse_on_surface: "#edf1f4", inverse_primary: "#6ad4f7", inverse_surface: "#2c3133", on_cold: "#ffffff", on_cold_container: "#ffffff", on_error: "#ffffff", on_error_container: "#410002", on_primary: "#ffffff", on_primary_container: "#ffffff", on_secondary: "#ffffff", on_secondary_container: "#274b58", on_surface: "#171c1e", on_surface_variant: "#3e484d", on_tertiary: "#ffffff", on_tertiary_container: "#ffffff", outline: "#6e797e", outline_variant: "#bdc8ce", primary: "#00647c", primary_container: "#007f9c", scrim: "#000000", secondary: "#406370", secondary_container: "#c0e6f5", shadow: "#000000", surface: "#f6fafd", surface_bright: "#f6fafd", surface_container: "#eaeef1", surface_container_high: "#e5e9eb", surface_container_highest: "#dfe3e6", surface_container_low: "#f0f4f7", surface_container_lowest: "#ffffff", surface_dim: "#d6dbdd", surface_variant: "#d9e4ea", tertiary: "#8b4c00", tertiary_container: "#aa631a" },
  },
  {
    slug: "moss",
    name: "Moss",
    seed: "#6a9a2e",
    dark: { cold: "#ffade6", cold_container: "#d26db8", error: "#ffb4ab", error_container: "#93000a", inverse_on_surface: "#2e3229", inverse_primary: "#406900", inverse_surface: "#e1e4d7", on_cold: "#5e0151", on_cold_container: "#000000", on_error: "#690005", on_error_container: "#ffdad6", on_primary: "#1f3700", on_primary_container: "#000000", on_secondary: "#24350e", on_secondary_container: "#d4ecb3", on_surface: "#e1e4d7", on_surface_variant: "#c3c9b5", on_tertiary: "#5e0151", on_tertiary_container: "#000000", outline: "#8d9381", outline_variant: "#434939", primary: "#a2d663", primary_container: "#6e9e32", scrim: "#000000", secondary: "#b7ce98", secondary_container: "#3c4f24", shadow: "#000000", surface: "#11140d", surface_bright: "#373a31", surface_container: "#1d2119", surface_container_high: "#282b23", surface_container_highest: "#33362d", surface_container_low: "#191d15", surface_container_lowest: "#0c0f08", surface_dim: "#11140d", surface_variant: "#434939", tertiary: "#ffade6", tertiary_container: "#d26db8" },
    light: { cold: "#943880", cold_container: "#b1519a", error: "#ba1a1a", error_container: "#ffdad6", inverse_on_surface: "#f0f2e5", inverse_primary: "#a2d663", inverse_surface: "#2e3229", on_cold: "#ffffff", on_cold_container: "#ffffff", on_error: "#ffffff", on_error_container: "#410002", on_primary: "#ffffff", on_primary_container: "#ffffff", on_secondary: "#ffffff", on_secondary_container: "#3b4e23", on_surface: "#191d15", on_surface_variant: "#434939", on_tertiary: "#ffffff", on_tertiary_container: "#ffffff", outline: "#737968", outline_variant: "#c3c9b5", primary: "#3e6700", primary_container: "#538115", scrim: "#000000", secondary: "#516538", secondary_container: "#d3ebb2", shadow: "#000000", surface: "#f9fbed", surface_bright: "#f9fbed", surface_container: "#edefe2", surface_container_high: "#e7e9dc", surface_container_highest: "#e1e4d7", surface_container_low: "#f3f5e7", surface_container_lowest: "#ffffff", surface_dim: "#d9dbce", surface_variant: "#dfe5d0", tertiary: "#943880", tertiary_container: "#b1519a" },
  },
];
