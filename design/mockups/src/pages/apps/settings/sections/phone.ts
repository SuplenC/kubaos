// Phone: the phone link (`kubaos-phone`, KDE Connect behind the switch file
// `desktop/phone/phone-link`), the phone paired with it, and what the two share. Only what the link
// is for works out of the box; the rest of KDE Connect waits behind the `plugins` file.

import { USER } from "../../../kobo/setup-cards.ts";
import type { Setting, SettingsSection, SubPage } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { fact, opens, toggle } from "./kit.ts";
import { dialogOver } from "../../../../lib/dialog.ts";
import { ADDRESS, counted, DEFAULTS, forgetQuestion, nameProblem, nonePaired, pairingPart, type Phone, phoneLine, type PhoneValues, preferred, redraw, rowAction } from "./phone-parts.ts";

function folderProblem(value: string): string | null {
  return /^~?\//.test(value.trim()) ? null : "A folder from ~/ or /, as ~/Downloads.";
}

/** A plugin beyond the link's own, allowed by its line in `plugins`. */
function allowed(v: PhoneValues, key: "notifications" | "touchpad" | "media" | "commands", id: string, title: string, line: string, more: Partial<Setting> = {}): Setting {
  return toggle(id, title, line, () => v[key], (on) => (v[key] = on), { initial: () => DEFAULTS[key], ...more });
}

/** How many plugins are allowed beyond the link's own, as a row's detail says it. */
function allowance(v: PhoneValues): string {
  const count = [v.notifications, v.touchpad, v.media, v.commands].filter(Boolean).length;
  if (count === 0) return "The link's own";
  return count === 1 ? "1 more" : `${count} more`;
}

/** Phone, whole. */
export function phone(values: Values): SettingsSection {
  const v = own(values, "phone", () => ({ ...DEFAULTS }));

  /** A paired phone's row on Paired phones: how it is, and Forget. */
  const pairedRow = (each: Phone): Setting => ({
    id: `paired-${each.id}`,
    title: each.name,
    line: phoneLine(each),
    control: {
      kind: "part",
      make: () =>
        rowAction("Forget", undefined, () => {
          if (!values.window) return;
          dialogOver(values.window, forgetQuestion(each), (answer) => {
            if (answer !== "Forget") return;
            v.phones = v.phones.filter((one) => one.id !== each.id);
            redraw(values);
          });
        }),
    },
  });

  const paired: SubPage = {
    id: "paired",
    title: "Paired phones",
    body: "Every phone this laptop trusts. Forget one and it has to pair again.",
    back: "Your phone",
    before: () => nonePaired(v),
    // A getter: a phone forgotten leaves the list as it is drawn again.
    get wells() {
      return v.link && v.phones.length ? [v.phones.map(pairedRow)] : [];
    },
  };

  const may: SubPage = {
    id: "may",
    title: "What the phone may do",
    body: "Beyond the clipboard and files, nothing, until you allow it here.",
    back: "Sharing",
    wells: [
      [fact("link-own", "The link's own", "The clipboard, files and text, ringing, battery and signal.", "Always")],
      [
        allowed(v, "notifications", "phone-notifications", "Its notifications here", "The phone's notifications land in your panel too. Android only."),
        allowed(v, "media", "media", "Media controls", "Play, pause and skip what plays here, from the phone."),
        allowed(v, "touchpad", "touchpad", "Touchpad and keyboard", "Move the pointer and type here from the phone.", {
          warns: () => (v.touchpad ? { title: "The phone can type on this laptop.", body: "So can anyone holding it unlocked." } : null),
        }),
        allowed(v, "commands", "commands", "Run commands", "The commands set in KDE Connect, run as you from the phone.", {
          warns: () => (v.commands ? { title: "The phone can run commands as you.", body: "Only the ones listed in KDE Connect, but with all your rights." } : null),
        }),
      ],
    ],
  };

  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "smartphone",
    title: "Phone",
    accent: "電話",
    lede: "Your other screen, on a short leash.",
    subs: [
      {
        title: "Link",
        body: "Off until you set it up: nothing listens for phones till then.",
        wells: [
          [
            toggle("phone-link", "Phone link", "Shares the clipboard and files with your phone, over KDE Connect.", () => v.link, (on) => (v.link = on), { drives: true, initial: () => DEFAULTS.link }),
            {
              id: "shown-name",
              title: "Shows up as",
              line: "The name your phone lists this laptop by.",
              control: { kind: "field", value: () => v.name, placeholder: USER.host, check: nameProblem, set: (value) => (v.name = value) },
              initial: () => DEFAULTS.name,
              unbuilt: true,
            },
            fact("address", "Address", "Add the laptop by this in the phone's app if it can't find it.", ADDRESS, { shows: () => v.link }),
          ],
        ],
      },
      {
        title: "Your phone",
        body: "Pair it once; after that they find each other.",
        before: () => pairingPart(values, v),
        wells: [
          [
            {
              id: "phone-now",
              // A getter: the phone it names is the one in reach as the row is drawn.
              get title() {
                return preferred(v)?.name ?? "";
              },
              get line() {
                const shown = preferred(v);
                return shown ? phoneLine(shown) : "";
              },
              control: {
                kind: "part",
                make: () => {
                  const shown = preferred(v);
                  return rowAction("Ring", "volume_up", () => {}, !shown?.reachable);
                },
              },
              shows: () => v.link && v.phones.length > 0,
            },
            opens("paired-phones", "Paired phones", "Every phone this laptop trusts; forget one there.", "paired", () => counted(v), { shows: () => v.link }),
            fact("paired-off", "Paired phones", "", "Off", { shows: () => !v.link, disabled: "Turn the phone link on to see them." }),
          ],
        ],
      },
      {
        title: "Sharing",
        body: "What goes between them, and what never does.",
        wells: [
          [
            toggle("send-copies", "Send what you copy", "Copy here and it's on the phone's clipboard too.", () => v.sendCopies, (on) => (v.sendCopies = on), { drives: true, initial: () => DEFAULTS.sendCopies, unbuilt: true }),
            toggle("passwords", "Passwords too", "Copies a password manager marks as secret go across as well.", () => v.passwords, (on) => (v.passwords = on), { shows: () => v.sendCopies, initial: () => DEFAULTS.passwords, unbuilt: true }),
            {
              id: "inbox",
              title: "Received files go to",
              line: "Where a file or photo the phone shares lands.",
              control: { kind: "field", value: () => v.inbox, placeholder: DEFAULTS.inbox, check: folderProblem, set: (value) => (v.inbox = value) },
              initial: () => DEFAULTS.inbox,
              unbuilt: true,
            },
          ],
          [opens("may", "What the phone may do", "Its notifications, a touchpad, media, commands: each off until allowed.", "may", () => allowance(v))],
        ],
      },
    ],
    pages: [paired, may],
  };
}
