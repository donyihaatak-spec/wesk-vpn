// Хук управляет состоянием списка конфигураций и операциями над ними.
// Вся логика общения с бэкендом инкапсулирована здесь; UI-компоненты лишь
// вызывают возвращаемые функции и реагируют на состояние.

import { useCallback, useEffect, useState } from "react";

import {
  deleteConfig,
  importConfigFromPath,
  importConfigFromText,
  listConfigs,
  renameConfig,
  toMessage,
  type VpnConfigRecord,
} from "../lib/tauri";

export interface UseConfigsResult {
  configs: VpnConfigRecord[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  importFromPath: (path: string) => Promise<VpnConfigRecord>;
  importFromText: (name: string, content: string) => Promise<VpnConfigRecord>;
  rename: (id: string, name: string) => Promise<VpnConfigRecord>;
  remove: (id: string) => Promise<void>;
}

export function useConfigs(): UseConfigsResult {
  const [configs, setConfigs] = useState<VpnConfigRecord[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      setConfigs(await listConfigs());
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Мутации намеренно пробрасывают ошибку наверх: их показывает UI
  // (через уведомления), а список перечитывается при успехе.
  const importFromPath = useCallback(
    async (path: string): Promise<VpnConfigRecord> => {
      const record = await importConfigFromPath(path);
      await refresh();
      return record;
    },
    [refresh],
  );

  const importFromText = useCallback(
    async (name: string, content: string): Promise<VpnConfigRecord> => {
      const record = await importConfigFromText(name, content);
      await refresh();
      return record;
    },
    [refresh],
  );

  const rename = useCallback(
    async (id: string, name: string): Promise<VpnConfigRecord> => {
      const record = await renameConfig(id, name);
      await refresh();
      return record;
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string): Promise<void> => {
      await deleteConfig(id);
      await refresh();
    },
    [refresh],
  );

  return {
    configs,
    loading,
    error,
    refresh,
    importFromPath,
    importFromText,
    rename,
    remove,
  };
}
