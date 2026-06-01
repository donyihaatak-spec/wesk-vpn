/** Лёгкий полноэкранный эффект при подключении / успехе. */

import type { ProxyStatus } from "../lib/tauri";

interface ConnectAmbientProps {
  state: ProxyStatus["state"];
}

export function ConnectAmbient({ state }: ConnectAmbientProps) {
  const show =
    state === "connecting" || state === "disconnecting" || state === "connected";

  if (!show) return null;

  return (
    <div className={`connect-ambient connect-ambient--${state} connect-ambient--on`} aria-hidden>
      {state === "connecting" && (
        <>
          <span className="connect-ambient__ring connect-ambient__ring--1" />
          <span className="connect-ambient__ring connect-ambient__ring--2" />
          {[0, 1, 2, 3, 4, 5].map((i) => (
            <span key={i} className={`connect-ambient__spark connect-ambient__spark--${i}`} />
          ))}
        </>
      )}
      {state === "connected" && (
        <>
          <span className="connect-ambient__burst connect-ambient__burst--1" />
          <span className="connect-ambient__burst connect-ambient__burst--2" />
        </>
      )}
    </div>
  );
}
