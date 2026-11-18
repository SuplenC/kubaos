// Settings › Proxy: the work network's VPN and where it stands, the kill switch, the work DNS, and
// the company's web proxy and certificate, from `proxy.toml` and `kubaos-proxy status`. Its own
// samples are the status part's states, the profile's refusals and its risky values.

import { screen } from "../../../site/page.ts";
import type { ProxyValues } from "./sections/proxy-parts.ts";
import { PROXY } from "./settings-data.ts";
import { sectionPage } from "./settings-section-page.ts";
import { litOn, settingsWindow } from "./settings-window.ts";

/** A WireGuard full tunnel, on, with the work DNS and the proxy set: what the live window opens with. */
const SET_UP: Partial<ProxyValues> = {
  vpn: "wireguard",
  state: "on",
  dnsServers: "10.0.0.53",
  dnsDomains: "corp.example.com",
  httpProxy: "http://proxy.corp.example.com:3128",
};

/** The section's own values, the set-up profile's as `more` changes them. */
const proxyWith = (more: Partial<ProxyValues> = {}) => ({ seed: { proxy: { ...SET_UP, ...more } } });

const at = (more: Partial<ProxyValues>) => screen(() => settingsWindow({ section: PROXY, values: proxyWith(more) }));
const lit = (id: string, more: Partial<ProxyValues>) => screen(() => settingsWindow(litOn(PROXY, id, proxyWith(more))));

export default sectionPage({
  from: ["button-bar", "card", "empty-state", "spinner"],
  index: PROXY,
  slug: "settings-proxy",
  title: "Settings: Proxy",
  lede: "The work network: IT's VPN and where it stands, its kill switch, the work DNS, the proxy and certificate.",
  hint: "switch it on; type ~/Downloads/work.ovpn",
  seed: proxyWith(),
  variants: [
    { name: "No VPN yet", for: "no profile: the empty state, and the import alone", sample: screen(() => settingsWindow({ section: PROXY })) },
    { name: "Connecting", for: "the spinner in its State row", sample: at({ state: "connecting" }) },
    { name: "Connected, WireGuard", for: "the live rows, a handshake among them", sample: at({}) },
    { name: "Connected, OpenVPN", for: "no handshake row", sample: at({ vpn: "openvpn" }) },
    { name: "Off", for: "the steady rows alone", sample: at({ state: "off" }) },
    { name: "Dropped, the kill switch holding", for: "a Warning banner over the card: off, or try again", sample: at({ state: "dropped" }) },
    { name: "Import refused: TAP", for: "the field red, the line says why", sample: lit("import", { importing: "~/Downloads/office-tap.ovpn" }) },
    { name: "Import refused: PKCS#12", for: "what to ask IT for instead", sample: lit("import", { importing: "~/Downloads/bundle.ovpn" }) },
    { name: "A proxy with a password", for: "refused: any program could read it", sample: lit("http-proxy", { httpProxy: "http://kuba:hunter2@proxy.corp.example.com:3128" }) },
    { name: "Kill switch off", for: "the warning under Safety's well", sample: lit("kill-switch", { killSwitch: false }) },
    { name: "Local network blocked", for: "a strict kill switch: what stops, under the well", sample: lit("allow-local", { allowLocal: false }) },
    { name: "A split tunnel", for: "no kill switch to have: the row dimmed, saying why", sample: lit("kill-switch-split", { full: false }) },
    { name: "Work DNS without a VPN", for: "a proxy alone: the DNS row dimmed, saying why", sample: lit("dns-needs-vpn", { vpn: "none", state: "off", dnsServers: "", dnsDomains: "" }) },
  ],
  measures: [
    { part: "The status card's rows", token: "space-s4" },
    { part: "Banner to the card", token: "space-s16" },
  ],
});
