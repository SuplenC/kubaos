# WeatherSky and its cards

Part of the closed set of components: [DESIGN.md, "Components"](../../DESIGN.md#components) has the rules every component keeps, its variant words and the fixed measures the geometry check holds it to.

Their mockup page is the source of truth: Molecules, WeatherSky, in `design/mockups/` (`src/lib/weather-sky.ts`, `src/lib/weather-card.ts`). The clock popup mounts the cards and lays them out.

## `WeatherSky`

A sky for an hour and a weather, drawn live: one fragment shader for its colours through the day, the sun, the moon in its phase, the stars, two cloud decks, haze, smog, fog and lightning. The weather is one of nine (clear, fair, partly cloudy, overcast, fog, rain, thunderstorm, snow, hail), with its readings: wind, visibility, UV index, humidity, air quality, feels like, rain in mm/h, snow in cm/h and hail slight or heavy, each its weather's typical value unless the forecast gives it. Every value the shader draws eases toward the one the hour and readings make, so a change of weather blends.

What falls is the forecast's amount: rain, snow or hail falls across the card (`atmos-fx`, MIT) and lands on the tops of the figures and words the owner names, each given an invisible box fitted to its ink. Lightning follows the weather, not the amount: a thunderstorm at 0.5 mm/h still flashes, rain at 30 mm/h doesn't.

- **Live** draws only while it is on screen and the page is shown; off screen it stops altogether.
- **Still** is one frame: nothing falls, nothing moves. A city's card.
- **Reduced motion** is a still sky with nothing falling and no lightning; the speed setting scales its clock.

The sky's colours are a picture's (a table by the hour), never interface colours, and no colour role is routed through them. Every sky on a page draws in one WebGL context.

## `HomeWeatherCard`, `WeatherBanner`, `CityCard` and `CityRow`

Cards that lie on a WeatherSky, each its own height (`size.weather_card` 112, `size.weather_banner` 132, `size.city_card` 92) and as wide as the room its owner gives it.

**Words on a sky** are `on_picture`, white in both schemes (the sky is the same picture by day and by night), with two shadows of `shadow` tight to the letters, at `opacity.picture_text_edge` and `picture_text_drop` (blurs of 1.5 and 1 px, the second half a pixel down). Nothing else lies over the picture: no fade, no scrim. **They are set in by their ink:** the large figures' ink stands on the small line's baseline, and that baseline stands as far above the card's bottom edge as the figures' ink stands in from its left: `space.s16` on home's card and the banner, `space.s12` on a city's.

- **HomeWeatherCard** (`content` radius): home's weather on a live sky, the temperature in `hero_number` and beside it, `space.s16` away, the condition (`subheading`) over "place · high / low" (`small`); what falls lands on its words; a press opens home's weather. Offline, the last reading dimmed (`opacity.placeholder`) with its age after the line ("2 h ago"); offline past a day, "No connection" over the place on the plain surface (`surface_container`); with no home city, "Set your city" on the plain surface.
- **WeatherBanner** (`content` radius): a city's weather on a live sky at the head of the weather view, laid out as home's card with "Feels like X° · high / low" under the condition.
- **CityCard** (`control` radius): another city on a still sky: its name bottom left (`body`, one line, cut with an ellipsis) over its time (`headline`); the offset ("+8 h", `small`) bottom right on the time's baseline; the temperature (`heading`, a step under the time) top right, its ink `space.s12` from the top and the right. A press opens that city's weather.
- **CityRow**: one to eight city cards, `space.s8` apart: one to four share one row, five to eight take a second row of the rest, each row's cards sharing its width equally, so a short last row stretches them. With no cities its owner draws no row.

**Carrying a sky.** One card's live sky can be carried into another card's place (`carry`): the clock popup's card growing into the weather view's banner and back. The one sky changes box, place, size and corners, drawing every frame as it goes, what falls included (its density holds while the box moves and settles once it lands), and stays as the sky of the card it lands as: a city's still sky comes alive for the move and stills again when it lands back. The words ride on top, never cut by the box: the temperature and the condition travel from their place and size on the one card to theirs on the other, on the box's curve; a piece only one card has fades as the move starts or as it lands.

*Not built yet: no library has them; they are mockups only. The tokens cannot yet hold the shadow's lengths under a whole pixel, so the mockup writes them out. A card has no lit or pressed look yet.*
