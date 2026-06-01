import { useCallback, useEffect, useState } from "react";

export interface AppSettings {
  /** Подключаться к последнему серверу при запуске. */
  autoConnectLast: boolean;
  /** Компактные карточки серверов. */
  compactCards: boolean;
  /** Меньше анимаций (accessibility). */
  reduceMotion: boolean;
  /** Показывать hero-панель статуса. */
  showHero: boolean;
}

const STORAGE_KEY = "wesk.settings";
export const LAST_PROFILE_KEY = "wesk.lastProfileId";

const DEFAULTS: AppSettings = {
  autoConnectLast: false,
  compactCards: false,
  reduceMotion: false,
  showHero: true,
};

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    return { ...DEFAULTS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULTS };
  }
}

function saveSettings(settings: AppSettings): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

export function useSettings() {
  const [settings, setSettingsState] = useState<AppSettings>(loadSettings);

  const setSettings = useCallback((patch: Partial<AppSettings>) => {
    setSettingsState((prev) => {
      const next = { ...prev, ...patch };
      saveSettings(next);
      return next;
    });
  }, []);

  useEffect(() => {
    document.documentElement.classList.toggle("reduce-motion", settings.reduceMotion);
    document.documentElement.classList.toggle("compact-cards", settings.compactCards);
  }, [settings.reduceMotion, settings.compactCards]);

  return { settings, setSettings };
}

export function saveLastProfileId(id: string): void {
  localStorage.setItem(LAST_PROFILE_KEY, id);
}

export function getLastProfileId(): string | null {
  return localStorage.getItem(LAST_PROFILE_KEY);
}
