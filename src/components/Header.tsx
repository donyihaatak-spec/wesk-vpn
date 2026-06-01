import { BRAND } from "../lib/brand";
import type { ProxyStatus } from "../lib/tauri";
import { Logo } from "./Logo";
import { StatusBadge, type BadgeTone } from "./StatusBadge";

interface HeaderProps {
  status: ProxyStatus;
  busy: boolean;
  onImport: () => void;
  onSettings: () => void;
}

const STATE_TONE: Record<ProxyStatus["state"], BadgeTone> = {
  disconnected: "muted",
  connecting: "info",
  connected: "ok",
  disconnecting: "info",
  error: "danger",
};

const STATE_SHORT: Record<ProxyStatus["state"], string> = {
  disconnected: "OFF",
  connecting: "…",
  connected: "ON",
  disconnecting: "…",
  error: "!",
};

export function Header({ status, busy, onImport, onSettings }: HeaderProps) {
  const transitioning =
    status.state === "connecting" || status.state === "disconnecting";

  return (
    <header className="header">
      <div className="header__brand">
        <Logo size={42} />
        <div className="header__titles">
          <h1 className="header__title">{BRAND.name}</h1>
          <p className="header__subtitle">{BRAND.protocolLine}</p>
        </div>
      </div>

      <div className="header__status header__status--compact">
        <StatusBadge tone={STATE_TONE[status.state]} pulse={transitioning}>
          {STATE_SHORT[status.state]}
        </StatusBadge>
      </div>

      <div className="header__actions">
        <button
          type="button"
          className="btn btn--ghost btn--icon-text"
          onClick={onSettings}
          disabled={busy}
          aria-label="Настройки"
        >
          <span className="btn__icon" aria-hidden>
            ⚙
          </span>
          <span className="btn__label">Настройки</span>
        </button>
        <button
          type="button"
          className="btn btn--primary btn--icon-text"
          onClick={onImport}
          disabled={busy}
          aria-label="Добавить ключ"
        >
          <span className="btn__icon" aria-hidden>
            +
          </span>
          <span className="btn__label">Добавить</span>
        </button>
      </div>
    </header>
  );
}
