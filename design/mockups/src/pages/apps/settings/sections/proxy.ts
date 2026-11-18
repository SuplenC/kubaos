// Proxy: the work network, from `proxy.toml` and `kubaos-proxy`: IT's VPN and where it stands, the
// kill switch that holds when it drops, the work DNS, and the company's web proxy and certificate.

import { button } from "../../../../lib/button.ts";
import type { Setting, SettingsSection } from "../settings-parts.ts";
import { own, type Values } from "../settings-values.ts";
import { fact, opens, toggle } from "./kit.ts";
import { configured, PROXY_DEFAULTS as D, type ProxyState, type ProxyValues, proxyStatus, type VpnType } from "./proxy-parts.ts";

/** How long the sample's VPN takes to come up: a network's pace, not a design value. */
const CONNECTS = 1500;

/** The files the sample's Downloads holds, and what `import` makes of each: a VPN, or why it refuses. */
const DOWNLOADS: Readonly<Record<string, { readonly vpn: VpnType; readonly full: boolean } | string>> = {
  "~/Downloads/work.conf": { vpn: "wireguard", full: true },
  "~/Downloads/office.conf": { vpn: "wireguard", full: false },
  "~/Downloads/work.ovpn": { vpn: "openvpn", full: true },
  "~/Downloads/office-tap.ovpn": "A TAP config: KubaOS's VPN runs TUN alone.",
  "~/Downloads/bundle.ovpn": "A PKCS#12 bundle: ask IT for the certificate and key as PEM.",
};

function importProblem(value: string): string | null {
  const path = value.trim();
  if (!/\.(conf|ovpn)$/.test(path)) return "A WireGuard .conf or an OpenVPN .ovpn file.";
  const found = DOWNLOADS[path];
  if (found === undefined) return "Nothing at that path.";
  return typeof found === "string" ? found : null;
}

/** The comma- or space-separated items of a list a field holds. */
const items = (value: string) => value.split(/[\s,]+/).filter((each) => each !== "");

function isAddress(item: string): boolean {
  const v4 = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/.exec(item);
  if (v4) return v4.slice(1).every((part) => Number(part) <= 255);
  return item.includes(":") && /^[0-9a-f:]+$/i.test(item);
}

const isDomain = (item: string) => /^(?=.{1,253}$)([a-z0-9-]+\.)+[a-z]{2,}$/i.test(item);

function serversProblem(value: string): string | null {
  return items(value).every(isAddress) ? null : "Addresses, as 10.0.0.53, a comma between.";
}

function domainsProblem(value: string): string | null {
  return items(value).every(isDomain) ? null : "Domains, as corp.example.com, a comma between.";
}

/** The proxy as `[http] proxy` takes it: `http://` or `https://`, a host and a port, nothing else. */
function proxyProblem(value: string): string | null {
  const url = value.trim();
  if (url.includes("@")) return "No user or password: any program could read them.";
  if (!/^https?:\/\//.test(url)) return "Starts with http:// or https://.";
  return /^https?:\/\/[a-z0-9.-]+:\d{1,5}$/i.test(url) ? null : "A host and a port, nothing else, as :3128.";
}

function pacProblem(value: string): string | null {
  return /^https?:\/\/[^\s"'\\]+$/.test(value.trim()) ? null : "An http:// or https:// address, no spaces.";
}

function aroundProblem(value: string): string | null {
  return items(value).every((item) => /^[a-z0-9.:/*-]+$/i.test(item)) ? null : "Hosts, domains or ranges, a comma between.";
}

function caProblem(value: string): string | null {
  return /\.(crt|pem)$/.test(value.trim()) ? null : "A PEM certificate, as company-ca.crt.";
}

function vaultProblem(value: string): string | null {
  return value.length <= 128 && !/\p{Cc}/u.test(value) ? null : "A vault item's name, up to 128 characters.";
}

/** A row holding one of `proxy.toml`'s text keys, checked as the profile checks it. */
function keyRow(p: ProxyValues, key: "dnsServers" | "dnsDomains" | "httpProxy" | "pac" | "noProxy" | "ca" | "vaultItem", id: string, title: string, line: string, placeholder: string, check: (value: string) => string | null, more: Partial<Setting> = {}): Setting {
  return { id, title, line, control: { kind: "field", value: () => p[key], placeholder, check, set: (value) => (p[key] = value) }, initial: () => D[key], ...more };
}

/** Proxy, whole. */
export function proxy(values: Values): SettingsSection {
  const p = own(values, "proxy", () => ({ ...D }));
  const redraw = () => values.window?.querySelector<HTMLElement>(".st-region")?.dispatchEvent(new CustomEvent("st-redraw"));
  const turn = (state: ProxyState) => {
    p.state = state;
    if (state === "connecting") {
      setTimeout(() => {
        if (p.state !== "connecting") return;
        p.state = "on";
        redraw();
      }, CONNECTS);
    }
  };
  const hasVpn = () => p.vpn !== "none";
  const holds = () => hasVpn() && p.full && p.killSwitch;
  return {
    ...(values.opened ? { opened: values.opened } : {}),
    icon: "vpn_lock",
    title: "Proxy",
    accent: "代理",
    lede: "Tunnel vision, the good kind.",
    subs: [
      {
        title: "VPN",
        body: "IT's VPN, the work DNS and the proxy, all up together.",
        before: () =>
          proxyStatus(p, (state) => {
            turn(state);
            redraw();
          }),
        wells: [
          [
            toggle("proxy", "Proxy", "Off at every login. The first time on asks for your password.", () => p.state !== "off", (on) => turn(on ? "connecting" : "off"), {
              drives: true,
              shows: () => configured(p),
              initial: () => D.state !== "off",
            }),
            {
              id: "import",
              title: "VPN config",
              line: "IT's .conf or .ovpn. Its keys go to your vault, never to disk.",
              control: {
                kind: "field",
                value: () => p.importing,
                placeholder: "~/Downloads/work.conf",
                check: importProblem,
                set: (path) => {
                  const found = DOWNLOADS[path.trim()];
                  if (typeof found !== "object") return;
                  p.vpn = found.vpn;
                  p.full = found.full;
                  p.importing = "";
                },
              },
              drives: true,
            },
          ],
          [
            keyRow(p, "vaultItem", "vault-item", "Vault item", "The Bitwarden note the VPN's keys are kept in.", "kubaos-proxy", vaultProblem),
            {
              id: "pull",
              title: "Keys on a new machine",
              line: "Copies the VPN's keys from your vault into this keyring.",
              control: { kind: "part", make: () => button({ label: "Copy from vault", kind: "Tonal", size: "Sm", icon: "key" }) },
            },
          ],
        ],
      },
      {
        title: "Safety",
        body: "What holds when the VPN drops.",
        wells: [
          [
            toggle("kill-switch", "Kill switch", "While the VPN is down, nothing leaves outside it. DNS included.", () => p.killSwitch, (on) => (p.killSwitch = on), {
              drives: true,
              shows: () => !hasVpn() || p.full,
              initial: () => D.killSwitch,
              warns: () => (p.killSwitch ? null : { title: "Nothing holds traffic back when the VPN drops.", body: "If it can't come back, the Proxy goes off and apps go straight out." }),
            }),
            fact("kill-switch-split", "Kill switch", "", "Full tunnel only", {
              shows: () => hasVpn() && !p.full,
              disabled: "A split tunnel carries IT's ranges alone; the rest goes out as ever.",
            }),
            toggle("allow-local", "Allow local network", "The phone link, printers, casting and SSH here keep working.", () => p.allowLocal, (on) => (p.allowLocal = on), {
              drives: true,
              shows: () => !hasVpn() || holds(),
              initial: () => D.allowLocal,
              warns: () => (p.allowLocal ? null : { title: "Nothing on this network answers while the VPN is up.", body: "The phone link, printers, casting and SSH to a machine here stop until the Proxy is off." }),
            }),
            fact("local-ranges", "Lets through", "The network you're on, read again each time it changes.", "192.168.1.0/24", { shows: () => holds() && p.allowLocal }),
            fact("drops", "When it drops", "It comes back by itself once the network is up.", "Reconnects", { shows: hasVpn }),
          ],
        ],
      },
      {
        title: "Work DNS",
        body: "Names only the work network knows, asked of its own servers.",
        wells: [
          [
            keyRow(p, "dnsServers", "dns-servers", "DNS servers", "The work servers, reached through the VPN.", "10.0.0.53", serversProblem, { shows: hasVpn }),
            keyRow(p, "dnsDomains", "dns-domains", "Work domains", "Only these names ask the work servers; the rest ask your network's.", "corp.example.com", domainsProblem, { shows: hasVpn }),
            fact("dns-needs-vpn", "Work DNS", "", "Needs the VPN", { shows: () => !hasVpn(), disabled: "The work servers are reached through the VPN: bring in its config first." }),
          ],
        ],
      },
      {
        title: "Web proxy",
        body: "The company's proxy and certificate, taken at the next Proxy on.",
        wells: [
          [
            keyRow(p, "httpProxy", "http-proxy", "Proxy server", "Apps, apt, Docker, Nix and git go through it. Asks for your password once.", "http://proxy.corp.example.com:3128", proxyProblem),
            keyRow(p, "pac", "pac", "Proxy script (PAC)", "For GNOME and Chrome alone; the rest need the server.", "http://wpad.corp.example.com/proxy.pac", pacProblem),
            keyRow(p, "noProxy", "no-proxy", "Goes around it", "Hosts and ranges that skip it. localhost always does.", ".corp.example.com, 10.0.0.0/8", aroundProblem),
          ],
          [
            keyRow(p, "ca", "ca", "Company certificate", "IT's root, trusted by the system and Chrome while the Proxy is on.", "company-ca.crt", caProblem),
            opens("reaches", "Where they're set", "The apps and tools the proxy and the certificate reach.", "reaches", () => "9 places"),
          ],
        ],
      },
    ],
    pages: [
      {
        id: "reaches",
        title: "Where they're set",
        body: "What the proxy and the certificate reach while the Proxy is on. Off puts back what you had.",
        back: "Web proxy",
        wells: [
          [
            fact("reach-session", "New terminals and services", "They read http_proxy. One already open keeps what it had.", "Server"),
            fact("reach-gnome", "GNOME and its browsers", "GNOME's own proxy setting, which Firefox follows.", "Server or PAC"),
            fact("reach-chrome", "Chrome", "Through Chrome's policy.", "Server or PAC"),
            fact("reach-apt", "apt", "Updates fetch through it.", "Server"),
            fact("reach-docker", "Docker", "The daemon restarts to take it, so running containers stop.", "Server"),
            fact("reach-nix", "Nix", "Its daemon restarts to take it.", "Server"),
            fact("reach-git", "git", "Fetches and pushes over HTTP.", "Server"),
          ],
          [
            fact("reach-store", "The system's trust store", "curl, git, Nix, Docker and most tools read it.", "Certificate"),
            fact("reach-chrome-ca", "Chrome's trust", "Through Chrome's policy.", "Certificate"),
          ],
        ],
      },
    ],
  };
}
