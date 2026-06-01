import { useId } from "react";

import { BRAND } from "../lib/brand";
import type { ProxyStatus } from "../lib/tauri";
import { ConnectProgress } from "./ConnectProgress";
import { StatusBadge, type BadgeTone } from "./StatusBadge";

interface ConnectionHeroProps {
  status: ProxyStatus;
  profileCount: number;
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

const STATE_TONE: Record<ProxyStatus["state"], BadgeTone> = {
  disconnected: "muted",
  connecting: "info",
  connected: "ok",
  disconnecting: "info",
  error: "danger",
};

function ShieldSvg({
  gradId,
  gradOkId,
  mode,
}: {
  gradId: string;
  gradOkId: string;
  mode: "idle" | "busy" | "live";
}) {
  const stroke = mode === "live" ? `url(#${gradOkId})` : `url(#${gradId})`;
  const accent = mode === "busy" ? "#a78bfa" : `url(#${gradId})`;

  return (
    <svg className="hero-visual__shield" viewBox="0 0 64 72" fill="none">
      <defs>
        <linearGradient id={gradId} x1="8" y1="8" x2="56" y2="64">
          <stop offset="0%" stopColor="#ff6b4a" />
          <stop offset="45%" stopColor="#f43f8f" />
          <stop offset="100%" stopColor="#8b5cf6" />
        </linearGradient>
        <linearGradient id={gradOkId} x1="8" y1="8" x2="56" y2="64">
          <stop offset="0%" stopColor="#34d399" />
          <stop offset="100%" stopColor="#6ee7b7" />
        </linearGradient>
      </defs>
      <path
        d="M32 4 L54 14 V34 C54 50 44 62 32 68 C20 62 10 50 10 34 V14 Z"
        fill="rgba(255,255,255,0.04)"
        stroke={stroke}
        strokeWidth="2"
      />
      {mode === "live" ? (
        <path
          className="hero-visual__check"
          d="M22 36 L30 44 L44 28"
          stroke={`url(#${gradOkId})`}
          strokeWidth="3"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      ) : (
        <>
          <circle cx="32" cy="32" r="6" fill={mode === "busy" ? "#a78bfa" : accent} className="hero-visual__lock-dot" />
          <path
            d="M24 32 V28 C24 22.5 27.5 18 32 18 C36.5 18 40 22.5 40 28 V32"
            stroke={mode === "busy" ? "#a78bfa" : stroke}
            strokeWidth="2.5"
            strokeLinecap="round"
          />
          <rect
            x="22"
            y="32"
            width="20"
            height="14"
            rx="3"
            fill="rgba(18,18,28,0.9)"
            stroke={mode === "busy" ? "#a78bfa" : stroke}
            strokeWidth="2"
          />
        </>
      )}
    </svg>
  );
}

function HeroVisual({ state }: { state: ProxyStatus["state"] }) {
  const gradId = useId();
  const gradOkId = useId();
  const busy = state === "connecting" || state === "disconnecting";
  const live = state === "connected";

  const layerClass = (on: boolean) => `hero-visual__layer${on ? " hero-visual__layer--on" : ""}`;

  return (
    <div className={`hero-visual hero-visual--${state}`} aria-hidden>
      <span className="hero-visual__ring hero-visual__ring--a" />
      <span className="hero-visual__ring hero-visual__ring--b" />
      <span className="hero-visual__ring hero-visual__ring--c" />

      <div className={layerClass(!busy && !live)}>
        <div className="hero-visual__core">
          <ShieldSvg gradId={gradId} gradOkId={gradOkId} mode="idle" />
          <div className="hero-visual__bars hero-visual__bars--idle">
            {[0, 1, 2, 3].map((i) => (
              <span key={i} className={`hero-visual__bar hero-visual__bar--${i}`} />
            ))}
          </div>
        </div>
      </div>

      <div className={layerClass(busy)}>
        <span className="hero-visual__pulse-ring" />
        <span className="hero-visual__pulse-ring hero-visual__pulse-ring--2" />
        <svg className="hero-visual__spinner" viewBox="0 0 100 100">
          <circle className="hero-visual__spinner-track" cx="50" cy="50" r="46" fill="none" strokeWidth="3" />
          <circle
            className="hero-visual__spinner-arc"
            cx="50"
            cy="50"
            r="46"
            fill="none"
            strokeWidth="3"
            strokeLinecap="round"
          />
        </svg>
        <div className="hero-visual__core hero-visual__core--busy">
          <ShieldSvg gradId={gradId} gradOkId={gradOkId} mode="busy" />
          <div className="hero-visual__bars">
            {[0, 1, 2, 3].map((i) => (
              <span key={i} className={`hero-visual__bar hero-visual__bar--${i}`} />
            ))}
          </div>
        </div>
        {[0, 1, 2].map((i) => (
          <span key={i} className={`hero-visual__satellite hero-visual__satellite--${i}`} />
        ))}
      </div>

      <div className={layerClass(live)}>
        <div className="hero-visual__core hero-visual__core--live">
          <ShieldSvg gradId={gradId} gradOkId={gradOkId} mode="live" />
          <div className="hero-visual__bars">
            {[0, 1, 2, 3].map((i) => (
              <span key={i} className={`hero-visual__bar hero-visual__bar--${i}`} />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export function ConnectionHero({
  status,
  profileCount,
  busy,
  onImport,
  onDisconnect,
}: ConnectionHeroProps) {
  const connected = status.state === "connected";
  const connecting = status.state === "connecting";
  const disconnecting = status.state === "disconnecting";
  const transitioning = connecting || disconnecting;

  const title = connecting
    ? status.activeProfileName
      ? `Подключаем «${status.activeProfileName}»`
      : "Подключение…"
    : disconnecting
      ? "Отключение…"
      : connected && status.activeProfileName
        ? status.activeProfileName
        : profileCount > 0
          ? "Выберите сервер"
          : "Добавьте первый ключ";

  const hint = connecting
    ? "Поднимаем TUN-туннель и запускаем sing-box"
    : disconnecting
      ? "Закрываем туннель, подождите"
      : connected
        ? "Трафик идёт через зашифрованный туннель sing-box"
        : profileCount > 0
          ? "Нажмите «Подключить» на карточке ниже"
          : "VLESS, VMess или подписка провайдера";

  return (
    <section className={`hero hero--${status.state}`} aria-live="polite">
      <div className="hero__mesh" aria-hidden />
      <div className="hero__shine" aria-hidden />
      <div className={`hero__scanline${transitioning ? " hero__scanline--on" : ""}`} aria-hidden />

      <div className="hero__inner">
        <HeroVisual state={status.state} />

        <div className="hero__body hero__fade-target">
          <StatusBadge tone={STATE_TONE[status.state]} pulse={transitioning}>
            {STATE_LABEL[status.state]}
          </StatusBadge>

          <h2 className="hero__title" key={`title-${status.state}`}>
            {title}
          </h2>
          <p className="hero__hint" key={`hint-${status.state}`}>
            {hint}
          </p>

          <div className={`hero__progress-slot${transitioning ? " hero__progress-slot--open" : ""}`}>
            {transitioning && (
              <ConnectProgress mode={connecting ? "connecting" : "disconnecting"} />
            )}
          </div>

          <div
            className={`hero__extras${profileCount > 0 && !connected && !transitioning ? " hero__extras--on" : ""}`}
          >
            {profileCount > 0 && !connected && !transitioning && (
              <div className="hero__stats">
                <span className="hero__stat">
                  <strong>{profileCount}</strong>{" "}
                  {profileCount === 1 ? "сервер" : profileCount < 5 ? "сервера" : "серверов"}
                </span>
                <span className="hero__stat-divider" />
                <span className="hero__stat hero__stat--dim">{BRAND.protocolLine}</span>
              </div>
            )}
          </div>

          <div className={`hero__extras${connected && status.activeProfileName ? " hero__extras--on" : ""}`}>
            {connected && status.activeProfileName && (
              <div className="hero__live-pill">
                <span className="hero__live-dot" />
                Защищённое соединение активно
              </div>
            )}
          </div>
        </div>

        <div className="hero__actions hero__fade-target">
          <div className={`hero__action-pane${connected ? " hero__action-pane--on" : ""}`}>
            <button
              type="button"
              className="btn btn--ghost btn--lg hero__btn-disconnect"
              onClick={onDisconnect}
              disabled={busy}
            >
              Отключить
            </button>
          </div>
          <div
            className={`hero__action-pane${!connected && !transitioning ? " hero__action-pane--on" : ""}`}
          >
            <button
              type="button"
              className="btn btn--primary btn--lg hero__btn-add"
              onClick={onImport}
              disabled={busy || transitioning}
            >
              <span className="hero__btn-icon">+</span>
              {profileCount > 0 ? "Добавить сервер" : "Импорт ключа"}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}
