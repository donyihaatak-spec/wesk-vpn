import { useCallback, useEffect, useState } from "react";

import {
  connectProfile,
  deleteProfile,
  disconnectProfile,
  getProxyStatus,
  importProfileFromText,
  importSubscription,
  listProfiles,
  renameProfile,
  toMessage,
  type ProfileRecord,
  type ProxyStatus,
} from "../lib/tauri";

export interface UseProfilesResult {
  profiles: ProfileRecord[];
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  importKey: (text: string) => Promise<ProfileRecord>;
  importSub: (url: string) => Promise<ProfileRecord[]>;
  rename: (id: string, name: string) => Promise<ProfileRecord>;
  remove: (id: string) => Promise<void>;
}

export function useProfiles(): UseProfilesResult {
  const [profiles, setProfiles] = useState<ProfileRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      setProfiles(await listProfiles());
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const importKey = useCallback(
    async (text: string): Promise<ProfileRecord> => {
      const record = await importProfileFromText(text);
      await refresh();
      return record;
    },
    [refresh],
  );

  const importSub = useCallback(
    async (url: string): Promise<ProfileRecord[]> => {
      const records = await importSubscription(url);
      await refresh();
      return records;
    },
    [refresh],
  );

  const rename = useCallback(
    async (id: string, name: string): Promise<ProfileRecord> => {
      const record = await renameProfile(id, name);
      await refresh();
      return record;
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string): Promise<void> => {
      await deleteProfile(id);
      await refresh();
    },
    [refresh],
  );

  return { profiles, loading, error, refresh, importKey, importSub, rename, remove };
}

const INITIAL: ProxyStatus = {
  state: "disconnected",
  activeProfileId: null,
  activeProfileName: null,
  message: null,
};

export interface UseProxyStatusResult {
  status: ProxyStatus;
  setStatus: (s: ProxyStatus) => void;
  refresh: () => Promise<void>;
}

export function useProxyStatus(pollMs = 2500): UseProxyStatusResult {
  const [status, setStatus] = useState<ProxyStatus>(INITIAL);

  const refresh = useCallback(async (): Promise<void> => {
    try {
      setStatus(await getProxyStatus());
    } catch {
      /* ignore poll errors */
    }
  }, []);

  useEffect(() => {
    void refresh();
    const t = window.setInterval(() => void refresh(), pollMs);
    return () => window.clearInterval(t);
  }, [refresh, pollMs]);

  return { status, setStatus, refresh };
}

export async function connect(id: string): Promise<ProxyStatus> {
  return connectProfile(id);
}

export async function disconnect(): Promise<ProxyStatus> {
  return disconnectProfile();
}
