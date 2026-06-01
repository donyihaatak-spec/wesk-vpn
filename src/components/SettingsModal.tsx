import { useCallback, useEffect, useState } from "react";

import type { AppSettings } from "../hooks/useSettings";
import { BRAND } from "../lib/brand";
import {
  applySplitRules,
  detectProcesses,
  getAppDiagnostics,
  getSplitTunnelStatus,
  listSplitRules,
  resetTunAdapter,
  setSplitRuleEnabled,
  toMessage,
  type AppDiagnostics,
  type SplitRule,
  type SplitTunnelStatus,
} from "../lib/tauri";
import { Toggle } from "./Toggle";

type SettingsTab = "general" | "routing" | "system";

interface SettingsModalProps {
  settings: AppSettings;
  connected: boolean;
  onSettingsChange: (patch: Partial<AppSettings>) => void;
  onClose: () => void;
}

const MODE_LABEL: Record<SplitRule["mode"], string> = {
  includeVpn: "Через VPN",
  excludeVpn: "Мимо VPN",
};

function ruleLabel(rule: SplitRule): string {
  return (
    rule.domain ??
    rule.domainSuffix ??
    rule.processName ??
    rule.appPath ??
    "Правило"
  );
}

export function SettingsModal({
  settings,
  connected,
  onSettingsChange,
  onClose,
}: SettingsModalProps) {
  const [tab, setTab] = useState<SettingsTab>("general");
  const [rules, setRules] = useState<SplitRule[]>([]);
  const [tunnelStatus, setTunnelStatus] = useState<SplitTunnelStatus | null>(null);
  const [diagnostics, setDiagnostics] = useState<AppDiagnostics | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      const [r, st, diag] = await Promise.all([
        listSplitRules(),
        getSplitTunnelStatus(),
        getAppDiagnostics(),
      ]);
      setRules(r);
      setTunnelStatus(st);
      setDiagnostics(diag);
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const toggleRule = async (rule: SplitRule): Promise<void> => {
    setBusy(true);
    setError(null);
    try {
      const updated = await setSplitRuleEnabled(rule.id, !rule.enabled);
      setRules((prev) => prev.map((r) => (r.id === updated.id ? updated : r)));
      if (connected) {
        const st = await applySplitRules();
        setTunnelStatus(st);
      }
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const reapplyRules = async (): Promise<void> => {
    setBusy(true);
    setError(null);
    try {
      const st = await applySplitRules();
      setTunnelStatus(st);
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const resetTun = async (): Promise<void> => {
    setBusy(true);
    setError(null);
    try {
      await resetTunAdapter();
      setError(null);
      setCopied("tun");
      setTimeout(() => setCopied(null), 2500);
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const copyText = async (text: string, key: string): Promise<void> => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(key);
      setTimeout(() => setCopied(null), 2000);
    } catch {
      setError("Не удалось скопировать в буфер");
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal modal--settings"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal__head">
          <h2 className="modal__title" id="settings-title">
            Настройки
          </h2>
          <button type="button" className="modal__close" onClick={onClose} aria-label="Закрыть">
            ×
          </button>
        </div>

        <div className="settings-tabs">
          {(
            [
              ["general", "Общие"],
              ["routing", "Маршруты"],
              ["system", "Система"],
            ] as const
          ).map(([id, label]) => (
            <button
              key={id}
              type="button"
              className={`settings-tab${tab === id ? " settings-tab--active" : ""}`}
              onClick={() => setTab(id)}
            >
              {label}
            </button>
          ))}
        </div>

        <div className="settings-body">
          {loading && tab !== "general" ? (
            <p className="settings-loading">Загрузка…</p>
          ) : (
            <>
              {tab === "general" && (
                <section className="settings-section">
                  <h3 className="settings-section__title">Подключение</h3>
                  <Toggle
                    label="Авто-подключение"
                    hint="При запуске подключаться к последнему серверу"
                    checked={settings.autoConnectLast}
                    onChange={(v) => onSettingsChange({ autoConnectLast: v })}
                  />

                  <h3 className="settings-section__title">Интерфейс</h3>
                  <Toggle
                    label="Панель статуса"
                    hint="Большая карточка с состоянием VPN сверху"
                    checked={settings.showHero}
                    onChange={(v) => onSettingsChange({ showHero: v })}
                  />
                  <Toggle
                    label="Компактные карточки"
                    hint="Меньше отступов — больше серверов на экране"
                    checked={settings.compactCards}
                    onChange={(v) => onSettingsChange({ compactCards: v })}
                  />
                  <Toggle
                    label="Уменьшить анимации"
                    hint="Для слабых устройств и чувствительности к движению"
                    checked={settings.reduceMotion}
                    onChange={(v) => onSettingsChange({ reduceMotion: v })}
                  />
                </section>
              )}

              {tab === "routing" && (
                <section className="settings-section">
                  <div className="settings-section__head">
                    <div>
                      <h3 className="settings-section__title">Split tunnel</h3>
                      <p className="settings-section__desc">
                        Telegram через VPN, Госуслуги напрямую — и свои правила.
                      </p>
                    </div>
                    <button
                      type="button"
                      className="btn btn--ghost btn--sm"
                      disabled={busy}
                      onClick={() => void reapplyRules()}
                    >
                      Применить
                    </button>
                  </div>

                  {tunnelStatus && (
                    <div className="settings-stats">
                      <span>{tunnelStatus.rulesCount} правил</span>
                      <span>{tunnelStatus.applied ? "OS: активно" : "OS: не применено"}</span>
                      {connected && <span>VPN: подключён</span>}
                    </div>
                  )}

                  <ul className="rule-list">
                    {rules.map((rule) => (
                      <li key={rule.id} className="rule-item">
                        <div className="rule-item__info">
                          <span className="rule-item__name">{ruleLabel(rule)}</span>
                          <span className={`rule-item__mode rule-item__mode--${rule.mode}`}>
                            {MODE_LABEL[rule.mode]}
                          </span>
                        </div>
                        <input
                          type="checkbox"
                          className="toggle"
                          checked={rule.enabled}
                          disabled={busy}
                          aria-label={`${rule.enabled ? "Выключить" : "Включить"} ${ruleLabel(rule)}`}
                          onChange={() => void toggleRule(rule)}
                        />
                      </li>
                    ))}
                  </ul>
                </section>
              )}

              {tab === "system" && diagnostics && (
                <section className="settings-section">
                  <h3 className="settings-section__title">О приложении</h3>
                  <dl className="diag-list">
                    <div className="diag-row">
                      <dt>Название</dt>
                      <dd>{BRAND.name}</dd>
                    </div>
                    <div className="diag-row">
                      <dt>Версия</dt>
                      <dd>{diagnostics.version}</dd>
                    </div>
                    <div className="diag-row">
                      <dt>sing-box</dt>
                      <dd className={diagnostics.singboxPath ? "diag-ok" : "diag-warn"}>
                        {diagnostics.singboxPath ?? "Не найден"}
                      </dd>
                    </div>
                  </dl>

                  <h3 className="settings-section__title">Сеть</h3>
                  <p className="settings-section__desc">
                    Если подключение падает с «The object already exists» — сбросьте TUN-адаптер.
                  </p>
                  <button
                    type="button"
                    className="btn btn--ghost btn--sm settings-action"
                    disabled={busy}
                    onClick={() => void resetTun()}
                  >
                    {busy ? "…" : copied === "tun" ? "TUN сброшен" : "Сбросить TUN"}
                  </button>

                  <h3 className="settings-section__title">Пути</h3>
                  <div className="path-block">
                    <span className="path-block__label">Данные приложения</span>
                    <code className="path-block__path">{diagnostics.dataDir}</code>
                    <button
                      type="button"
                      className="btn btn--ghost btn--sm"
                      onClick={() => void copyText(diagnostics.dataDir, "data")}
                    >
                      {copied === "data" ? "Скопировано" : "Копировать"}
                    </button>
                  </div>
                  <div className="path-block">
                    <span className="path-block__label">Лог сети</span>
                    <code className="path-block__path">{diagnostics.networkLogPath}</code>
                    <button
                      type="button"
                      className="btn btn--ghost btn--sm"
                      onClick={() => void copyText(diagnostics.networkLogPath, "log")}
                    >
                      {copied === "log" ? "Скопировано" : "Копировать"}
                    </button>
                  </div>
                </section>
              )}
            </>
          )}

          {error && <div className="modal__error settings-error">{error}</div>}
        </div>
      </div>
    </div>
  );
}
