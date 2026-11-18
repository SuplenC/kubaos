// Proxy's own values and its organism: where the work network stands, read from `kubaos-proxy
// status`, drawn before the VPN's rows. Its rows look as the control center's Proxy detail draws
// them (`pages/shell/control-center-parts.ts`, `proxyDetail`), so the same readings look the same
// in both places.

import { banner } from "../../../../lib/banner.ts";
import { buttonBar } from "../../../../lib/button-bar.ts";
import { card } from "../../../../lib/card.ts";
import { emptyState } from "../../../../lib/empty-state.ts";
import { fill, filling, row, stack } from "../../../../lib/layout.ts";
import { spinner } from "../../../../lib/spinner.ts";
import { clamped, text } from "../../../../lib/text.ts";

/** Where the Proxy stands: `status --json`'s `wanted`, `on` and `busy`, and a VPN that can't come back. */
export type ProxyState = "off" | "connecting" | "on" | "dropped";

/** The VPN `import` brought in, by `[tunnel] type`; none before the first import. */
export type VpnType = "none" | "wireguard" | "openvpn";

/** The Proxy section's own values: `proxy.toml`'s keys, and the state `status` reads. */
export interface ProxyValues {
  vpn: VpnType;
  /** A full tunnel: every address goes through it, so the kill switch can hold. */
  full: boolean;
  state: ProxyState;
  /** What the import field holds: a path being typed, or one it refused. */
  importing: string;
  killSwitch: boolean;
  allowLocal: boolean;
  dnsServers: string;
  dnsDomains: string;
  httpProxy: string;
  pac: string;
  noProxy: string;
  ca: string;
  vaultItem: string;
}

/** KubaOS's defaults: no profile yet, and the keys' own defaults in `kubaos_proxy::profile`. */
export const PROXY_DEFAULTS: ProxyValues = {
  vpn: "none",
  full: true,
  state: "off",
  importing: "",
  killSwitch: true,
  allowLocal: true,
  dnsServers: "",
  dnsDomains: "",
  httpProxy: "",
  pac: "",
  noProxy: "",
  ca: "",
  vaultItem: "kubaos-proxy",
};

/** Whether the profile has any part to switch: the control center shows its tile from then on. */
export function configured(p: ProxyValues): boolean {
  return p.vpn !== "none" || p.httpProxy !== "" || p.pac !== "" || p.ca !== "";
}

/** The VPN as the status's rows name it: its type, the tunnel's kind, and the kill switch. */
export function vpnName(p: ProxyValues): string {
  const type = p.vpn === "openvpn" ? "OpenVPN" : "WireGuard";
  if (!p.full) return `${type}, split tunnel`;
  return p.killSwitch ? `${type}, full tunnel, kill switch` : `${type}, full tunnel`;
}

/** A label on the left and its value, in `mono`, on the right: the control center's detail row. */
function valueRow(label: string, value: string | HTMLElement): HTMLElement {
  return row("s12", "centre", text(label, "small", "on_surface_variant"), fill(), typeof value === "string" ? clamped(value, "mono") : value);
}

/** The state's row: a spinner beside it while the VPN comes up. */
function stateValue(state: ProxyState): string | HTMLElement {
  if (state === "connecting") return row("s8", "centre", spinner("s16"), clamped("Connecting…", "mono"));
  return { off: "Off", on: "On", dropped: "Dropped" }[state];
}

/** The readings a VPN that is up adds: when it last received, a WireGuard handshake, the bytes moved. */
function liveRows(p: ProxyValues): HTMLElement[] {
  if (p.state === "dropped") return [valueRow("Last received", "4 min ago")];
  if (p.state !== "on") return [];
  return [valueRow("Last received", "12 s ago"), ...(p.vpn === "wireguard" ? [valueRow("Last handshake", "32 s ago")] : []), valueRow("Moved", "48.2 MB in · 3.1 MB out")];
}

/** The server as the config names it, its port by type. */
function serverOf(p: ProxyValues): string {
  return p.vpn === "openvpn" ? "vpn.example.org:1194" : "vpn.example.org:51820";
}

/**
 * Where the Proxy stands, before the VPN's rows. No profile, an empty state saying how
 * to start; a VPN, a card of the status's rows; a VPN that dropped behind its kill switch, a
 * Warning banner over the card with what lifts it. `act` turns the Proxy off or on again from it.
 */
export function proxyStatus(p: ProxyValues, act: (state: ProxyState) => void): HTMLElement {
  if (!configured(p)) {
    return card({ body: [emptyState({ icon: "vpn_lock", title: "No VPN yet", body: "Bring in the config IT sent you, a WireGuard .conf or an OpenVPN .ovpn, below." })] });
  }
  const rows = [valueRow("State", stateValue(p.state)), ...(p.vpn === "none" ? [] : [valueRow("VPN", vpnName(p)), valueRow("Server", serverOf(p))]), ...liveRows(p)];
  const readings = card({ body: [stack("s4", "start", ...rows.map(filling))] });
  if (p.state !== "dropped") return readings;
  const bar = buttonBar({ size: "Sm", actions: [{ label: "Proxy off", kind: "Tonal" }, { label: "Try again" }] });
  bar.actions[0]!.addEventListener("click", () => act("off"));
  bar.actions[1]!.addEventListener("click", () => act("connecting"));
  const warning = banner({
    tone: "Warning",
    icon: "warning",
    title: "The VPN dropped. The kill switch holds.",
    body: "Nothing leaves outside the tunnel until it is back. The next network change tries again.",
    actions: bar.element,
  });
  return stack("s16", "start", filling(warning), filling(readings));
}
