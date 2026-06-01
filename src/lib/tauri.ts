// Единственный слой общения с Rust-бэкендом.

import { invoke } from "@tauri-apps/api/core";

// ─── Proxy-профили (Happ-style keys) ─────────────────────────────────────

export type ProxyProtocol =
  | "vless"
  | "vmess"
  | "trojan"
  | "shadowsocks"
  | "socks"
  | "hysteria2";

export interface ProfileRecord {
  id: string;
  name: string;
  protocol: ProxyProtocol;
  server: string;
  port: number;
  createdAt: string;
  updatedAt: string;
  rawUri: string;
}

export type ProxyState =
  | "disconnected"
  | "connecting"
  | "connected"
  | "disconnecting"
  | "error";

export interface ProxyStatus {
  state: ProxyState;
  activeProfileId: string | null;
  activeProfileName: string | null;
  message: string | null;
}

export function listProfiles(): Promise<ProfileRecord[]> {
  return invoke<ProfileRecord[]>("list_profiles");
}

export function importProfileFromText(text: string): Promise<ProfileRecord> {
  return invoke<ProfileRecord>("import_profile_from_text", { text });
}

export function importSubscription(url: string): Promise<ProfileRecord[]> {
  return invoke<ProfileRecord[]>("import_subscription", { url });
}

export function renameProfile(id: string, name: string): Promise<ProfileRecord> {
  return invoke<ProfileRecord>("rename_profile", { id, name });
}

export function deleteProfile(id: string): Promise<void> {
  return invoke<void>("delete_profile", { id });
}

export function connectProfile(id: string): Promise<ProxyStatus> {
  return invoke<ProxyStatus>("connect_profile", { id });
}

export function disconnectProfile(): Promise<ProxyStatus> {
  return invoke<ProxyStatus>("disconnect_profile");
}

export function getProxyStatus(): Promise<ProxyStatus> {
  return invoke<ProxyStatus>("get_proxy_status");
}

/** `null` если sing-box не найден. */
export function checkSingbox(): Promise<string | null> {
  return invoke<string | null>("check_singbox");
}

export function resetTunAdapter(): Promise<void> {
  return invoke<void>("reset_tun_adapter");
}

// ─── Split tunneling ───────────────────────────────────────────────────────

export type SplitMode = "includeVpn" | "excludeVpn";

export type VpnInterface = "tun0" | "wg0" | "system";

export interface SplitRule {
  id: string;
  appPath: string | null;
  processName: string | null;
  domain: string | null;
  domainSuffix: string | null;
  mode: SplitMode;
  interface: VpnInterface;
  priority: number;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  exePath: string | null;
}

export interface SplitTunnelStatus {
  rulesCount: number;
  applied: boolean;
  activeBackend: string | null;
  matchedProcesses: ProcessInfo[];
  networkLogPath: string;
}

export function listSplitRules(): Promise<SplitRule[]> {
  return invoke<SplitRule[]>("list_split_rules");
}

export function addSplitRule(params: {
  appPath?: string | null;
  processName?: string | null;
  domain?: string | null;
  domainSuffix?: string | null;
  mode: SplitMode;
  interface?: VpnInterface;
  priority?: number;
}): Promise<SplitRule> {
  return invoke<SplitRule>("add_split_rule", {
    appPath: params.appPath ?? null,
    processName: params.processName ?? null,
    domain: params.domain ?? null,
    domainSuffix: params.domainSuffix ?? null,
    mode: params.mode,
    interface: params.interface ?? "tun0",
    priority: params.priority ?? 0,
  });
}

export function removeSplitRule(id: string): Promise<void> {
  return invoke<void>("remove_split_rule", { id });
}

export function setSplitRuleEnabled(id: string, enabled: boolean): Promise<SplitRule> {
  return invoke<SplitRule>("set_split_rule_enabled", { id, enabled });
}

export function applySplitRules(): Promise<SplitTunnelStatus> {
  return invoke<SplitTunnelStatus>("apply_split_rules");
}

export function detectProcesses(): Promise<ProcessInfo[]> {
  return invoke<ProcessInfo[]>("detect_processes");
}

export function getSplitTunnelStatus(): Promise<SplitTunnelStatus> {
  return invoke<SplitTunnelStatus>("get_split_tunnel_status");
}

export interface AppDiagnostics {
  version: string;
  dataDir: string;
  singboxPath: string | null;
  networkLogPath: string;
}

export function getAppDiagnostics(): Promise<AppDiagnostics> {
  return invoke<AppDiagnostics>("get_app_diagnostics");
}

export const PROTOCOL_LABEL: Record<ProxyProtocol, string> = {
  vless: "VLESS",
  vmess: "VMess",
  trojan: "Trojan",
  shadowsocks: "Shadowsocks",
  socks: "SOCKS",
  hysteria2: "Hysteria2",
};

// ─── WireGuard (legacy) ────────────────────────────────────────────────────

export interface VpnConfigRecord {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  addresses: string[];
  dns: string[];
  endpoint: string | null;
  allowedIps: string[];
  peerCount: number;
  valid: boolean;
  raw: string;
}

export function toMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error) {
    const { message } = error as { message: unknown };
    if (typeof message === "string") return message;
  }
  return "Неизвестная ошибка";
}
