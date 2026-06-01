/** Не даёт UI мигнуть, если бэкенд ответил слишком быстро. */
export function minPhaseDuration(ms: number, startedAt: number): Promise<void> {
  const remaining = ms - (Date.now() - startedAt);
  if (remaining <= 0) return Promise.resolve();
  return new Promise((resolve) => window.setTimeout(resolve, remaining));
}

export const CONNECT_MIN_MS = 900;
export const DISCONNECT_MIN_MS = 650;
export const STATE_FADE_MS = 450;
