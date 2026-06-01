import { BRAND } from "../lib/brand";
import { PROTOCOL_LABEL, type ProfileRecord, type ProxyStatus } from "../lib/tauri";

interface SidebarProps {
  profiles: ProfileRecord[];
  status: ProxyStatus;
}

const STATE_HINT: Record<ProxyStatus["state"], string> = {
  disconnected: "Трафик идёт напрямую",
  connecting: "Поднимаем TUN-туннель…",
  connected: "Трафик шифруется через sing-box",
  disconnecting: "Закрываем туннель…",
  error: "Проверьте ключ или sing-box",
};

function protocolCounts(profiles: ProfileRecord[]): [string, number][] {
  const map = new Map<string, number>();
  for (const p of profiles) {
    const label = PROTOCOL_LABEL[p.protocol] ?? p.protocol;
    map.set(label, (map.get(label) ?? 0) + 1);
  }
  return [...map.entries()].sort((a, b) => b[1] - a[1]);
}

export function SidebarLeft({ profiles, status }: SidebarProps) {
  const connected = status.state === "connected";
  const transitioning = status.state === "connecting" || status.state === "disconnecting";

  return (
    <aside className="app-aside app-aside--left">
      <div className="aside-card aside-card--glow">
        <p className="aside-card__eyebrow">Туннель</p>
        <div className={`tunnel-viz tunnel-viz--${status.state}`}>
          <div className="tunnel-viz__node">
            <span className="tunnel-viz__icon">💻</span>
            <span className="tunnel-viz__label">Вы</span>
          </div>
          <div className="tunnel-viz__pipe">
            <span className="tunnel-viz__line" />
            {connected || transitioning ? <span className="tunnel-viz__packet" /> : null}
          </div>
          <div className="tunnel-viz__node">
            <span className="tunnel-viz__icon">🌐</span>
            <span className="tunnel-viz__label">Сеть</span>
          </div>
        </div>
        <p className="aside-card__hint">{STATE_HINT[status.state]}</p>
      </div>

      <div className="aside-card">
        <p className="aside-card__eyebrow">Split tunnel</p>
        <ul className="aside-list">
          <li>
            <span className="aside-list__dot aside-list__dot--ok" />
            Telegram, Discord → через VPN
          </li>
          <li>
            <span className="aside-list__dot aside-list__dot--dim" />
            Госуслуги, банки → напрямую
          </li>
        </ul>
      </div>
    </aside>
  );
}

export function SidebarRight({ profiles, status }: SidebarProps) {
  const connected = status.state === "connected";
  const counts = protocolCounts(profiles);
  const active = profiles.find((p) => p.id === status.activeProfileId);

  return (
    <aside className="app-aside app-aside--right">
      <div className="aside-card aside-card--stat">
        <p className="aside-stat__value">{profiles.length}</p>
        <p className="aside-stat__label">серверов</p>
      </div>

      {counts.length > 0 && (
        <div className="aside-card">
          <p className="aside-card__eyebrow">Протоколы</p>
          <ul className="aside-protocols">
            {counts.map(([name, n]) => (
              <li key={name} className="aside-protocols__row">
                <span>{name}</span>
                <span className="aside-protocols__count">{n}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {active && connected && (
        <div className="aside-card aside-card--active">
          <p className="aside-card__eyebrow">Активный</p>
          <p className="aside-active__name">{active.name}</p>
          <p className="aside-active__host">
            {active.server}:{active.port}
          </p>
        </div>
      )}

      <div className="aside-card aside-card--muted">
        <p className="aside-card__eyebrow">{BRAND.name}</p>
        <p className="aside-card__tagline">{BRAND.protocolLine}</p>
      </div>
    </aside>
  );
}
