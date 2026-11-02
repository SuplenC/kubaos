// Every mockup page, by level. A new page is a file under its level's folder and its line here.

import type { AnyPage, Level } from "../site/page.ts";
import badge from "./atoms/badge.ts";
import button from "./atoms/button.ts";
import checkbox from "./atoms/checkbox.ts";
import chip from "./atoms/chip.ts";
import divider from "./atoms/divider.ts";
import field from "./atoms/field.ts";
import figure from "./atoms/figure.ts";
import hint from "./atoms/hint.ts";
import icon from "./atoms/icon.ts";
import iconButton from "./atoms/icon-button.ts";
import keycap from "./atoms/keycap.ts";
import radio from "./atoms/radio.ts";
import skeleton from "./atoms/skeleton.ts";
import spinner from "./atoms/spinner.ts";
import toggle from "./atoms/switch.ts";
import text from "./atoms/text.ts";
import tooltip from "./atoms/tooltip.ts";
import colour from "./foundations/colour.ts";
import elevation from "./foundations/elevation.ts";
import focus from "./foundations/focus.ts";
import icons from "./foundations/icons.ts";
import motion from "./foundations/motion.ts";
import radii from "./foundations/radii.ts";
import space from "./foundations/space.ts";
import type from "./foundations/type.ts";
import layoutGrid from "./layout/grid.ts";
import layoutHeader from "./layout/header.ts";
import layoutPage from "./layout/page.ts";
import layoutStack from "./layout/stack.ts";
import badgedIcon from "./molecules/badged-icon.ts";
import buttonBar from "./molecules/button-bar.ts";
import calendar from "./molecules/calendar.ts";
import card from "./molecules/card.ts";
import choiceCard from "./molecules/choice-card.ts";
import collapsible from "./molecules/collapsible.ts";
import descriptionList from "./molecules/description-list.ts";
import dialog from "./molecules/dialog.ts";
import emptyList from "./molecules/empty-list.ts";
import iconTile from "./molecules/icon-tile.ts";
import list from "./molecules/list.ts";
import liveList from "./molecules/live-list.ts";
import media from "./molecules/media.ts";
import scrollingList from "./molecules/scrolling-list.ts";
import menu from "./molecules/menu.ts";
import pictureChoice from "./molecules/picture-choice.ts";
import pill from "./molecules/pill.ts";
import popover from "./molecules/popover.ts";
import scroll from "./molecules/scroll.ts";
import select from "./molecules/select.ts";
import slider from "./molecules/slider.ts";
import table from "./molecules/table.ts";
import tabs from "./molecules/tabs.ts";
import tile from "./molecules/tile.ts";
import toast from "./molecules/toast.ts";

export const PAGES: readonly AnyPage[] = [colour, type, space, radii, elevation, focus, motion, icons, layoutPage, layoutHeader, layoutStack, layoutGrid, text, icon, button, iconButton, hint, toggle, checkbox, radio, chip, field, keycap, tooltip, badge, spinner, divider, skeleton, figure, slider, tabs, select, popover, menu, calendar, tile, collapsible, buttonBar, dialog, toast, card, choiceCard, media, pictureChoice, iconTile, badgedIcon, pill, list, scrollingList, liveList, emptyList, table, descriptionList, scroll];

export function pagesOf(level: Level): AnyPage[] {
  return PAGES.filter((page) => page.level === level);
}
