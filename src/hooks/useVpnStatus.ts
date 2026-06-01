// Хук отслеживает статус VPN-подключения: периодически опрашивает бэкенд
// и позволяет оптимистично обновлять статус после connect/disconnect.

import { useCallback, useEffect, useState } from "react";

import { getVpnStatus, type VpnStatus } from "../lib/tauri";

const INITIAL_STATUS: VpnStatus = {
  state: "disconnected",
  activeConfigId: null,
  activeConfigName: null,
  message: null,
};

export interface UseVpnStatusResult {
  status: VpnStatus;
  setStatus: (status: VpnStatus) => void;
  refresh: () => Promise<void>;
}

export function useVpnStatus(pollMs = 2500): UseVpnStatusResult {
  const [status, setStatus] = useState<VpnStatus>(INITIAL_STATUS);

  const refresh = useCallback(async (): Promise<void> => {
    try {
      setStatus(await getVpnStatus());
    } catch {
      // Статус — вспомогательная информация: ошибку опроса не показываем,
      // чтобы не спамить уведомлениями. Действия пользователя обработают её.
    }
  }, []);

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => {
      void refresh();
    }, pollMs);
    return () => window.clearInterval(timer);
  }, [refresh, pollMs]);

  return { status, setStatus, refresh };
}
