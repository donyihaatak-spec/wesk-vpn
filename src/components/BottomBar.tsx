import type { ProxyStatus } from "../lib/tauri";

interface BottomBarProps {
  status: ProxyStatus;
  busy: boolean;
  onImport: () => void;
  onDisconnect: () => void;
}

const STATE_LABEL: Record<ProxyStatus["state"], string> = {
  disconnected: "Не подключено",
  connecting: "Подключение…",
  connected: "Защищено",
  disconnecting: "Отключение…",
  error: "Ошибка",
};

export function BottomBar({ status, busy, onImport, onDisconnect }: BottomBarProps) {
  const connected = status.state === "connected";
  const transitioning = status.state === "connecting" || status.state === "disconnecting";

  return (
    <nav className={`bottom-bar bottom-bar--${status.state}`} aria-label="Быстрые действия">
      <div className="bottom-bar__status">
        {transitioning && <span className="bottom-bar__pulse" aria-hidden />}
        <p className="bottom-bar__status-text">{STATE_LABEL[status.state]}</p>
        <p className="bottom-bar__status-sub">
          {status.activeProfileName ?? "Выберите сервер"}
        </p>
      </div>
      {connected ? (
        <button type="button" className="btn btn--ghost btn--sm" onClick={onDisconnect} disabled={busy}>
          Стоп
        </button>
      ) : (
        <button type="button" className="btn btn--primary btn--sm" onClick={onImport} disabled={busy}>
          + Ключ
        </button>
      )}
    </nav>
  );
}
