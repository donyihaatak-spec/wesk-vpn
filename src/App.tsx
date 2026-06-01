// Корневой компонент — управление proxy-профилями (ключи Happ).

import { useCallback, useEffect, useRef, useState } from "react";

import { AppBackdrop } from "./components/AppBackdrop";
import { AppFooterShowcase } from "./components/AppFooterShowcase";
import { BottomBar } from "./components/BottomBar";
import { ConnectAmbient } from "./components/ConnectAmbient";
import { ConnectionHero } from "./components/ConnectionHero";
import { SidebarLeft, SidebarRight } from "./components/DesktopSidebar";
import { Header } from "./components/Header";
import { ImportKeyModal } from "./components/ImportKeyModal";
import { ProfileList } from "./components/ProfileList";
import { SettingsModal } from "./components/SettingsModal";
import { useToast } from "./components/Toast";
import {
  connect,
  disconnect,
  useProfiles,
  useProxyStatus,
} from "./hooks/useProfiles";
import { getLastProfileId, saveLastProfileId, useSettings } from "./hooks/useSettings";
import { useDisplayStatus } from "./hooks/useDisplayStatus";
import { checkSingbox, resetTunAdapter, toMessage, type ProfileRecord } from "./lib/tauri";
import {
  CONNECT_MIN_MS,
  DISCONNECT_MIN_MS,
  minPhaseDuration,
} from "./lib/smoothTransition";

export default function App() {
  const toast = useToast();
  const { settings, setSettings } = useSettings();
  const { profiles, loading, error, refresh, importKey, importSub, rename, remove } =
    useProfiles();
  const { status, setStatus, refresh: refreshStatus } = useProxyStatus();
  const displayStatus = useDisplayStatus(status);

  const [importOpen, setImportOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [singboxPath, setSingboxPath] = useState<string | null | undefined>(undefined);
  const autoConnectAttempted = useRef(false);

  useEffect(() => {
    void checkSingbox().then(setSingboxPath);
  }, []);

  const handleConnect = useCallback(
    async (profile: ProfileRecord): Promise<void> => {
      const startedAt = Date.now();
      setBusy(true);
      setStatus({
        state: "connecting",
        activeProfileId: profile.id,
        activeProfileName: profile.name,
        message: null,
      });
      try {
        const next = await connect(profile.id);
        await minPhaseDuration(CONNECT_MIN_MS, startedAt);
        setStatus(next);
        saveLastProfileId(profile.id);
        toast.success(`Подключено: ${profile.name}`);
      } catch (e) {
        toast.error(toMessage(e));
        void refreshStatus();
      } finally {
        setBusy(false);
      }
    },
    [setStatus, refreshStatus, toast],
  );

  useEffect(() => {
    if (
      autoConnectAttempted.current ||
      loading ||
      !settings.autoConnectLast ||
      status.state !== "disconnected" ||
      profiles.length === 0
    ) {
      return;
    }

    const lastId = getLastProfileId();
    const profile = lastId ? profiles.find((p) => p.id === lastId) : undefined;
    if (!profile) return;

    autoConnectAttempted.current = true;
    void handleConnect(profile);
  }, [loading, profiles, settings.autoConnectLast, status.state, handleConnect]);

  const handleDisconnect = useCallback(async (): Promise<void> => {
    const startedAt = Date.now();
    setBusy(true);
    setStatus((prev) => ({
      ...prev,
      state: "disconnecting",
    }));
    try {
      const next = await disconnect();
      await minPhaseDuration(DISCONNECT_MIN_MS, startedAt);
      setStatus(next);
      toast.info("Отключено");
    } catch (e) {
      toast.error(toMessage(e));
      void refreshStatus();
    } finally {
      setBusy(false);
    }
  }, [setStatus, refreshStatus, toast]);

  const handleRename = useCallback(
    async (id: string, name: string): Promise<void> => {
      try {
        await rename(id, name);
        toast.success("Профиль переименован");
      } catch (e) {
        toast.error(toMessage(e));
        throw e;
      }
    },
    [rename, toast],
  );

  const handleDelete = useCallback(
    async (id: string): Promise<void> => {
      try {
        await remove(id);
        toast.info("Профиль удалён");
      } catch (e) {
        toast.error(toMessage(e));
      }
    },
    [remove, toast],
  );

  return (
    <div
      className={`app app--${displayStatus.state}${busy ? " app--busy" : ""}`}
    >
      <AppBackdrop />
      <ConnectAmbient state={displayStatus.state} />

      <Header
        status={displayStatus}
        busy={busy}
        onImport={() => setImportOpen(true)}
        onSettings={() => setSettingsOpen(true)}
      />

      <div className="app-shell">
        <SidebarLeft profiles={profiles} status={displayStatus} />

        <main className="main">
        {settings.showHero && (
          <ConnectionHero
            status={displayStatus}
            profileCount={profiles.length}
            busy={busy}
            onImport={() => setImportOpen(true)}
            onDisconnect={() => void handleDisconnect()}
          />
        )}
        {singboxPath === null && (
          <div className="banner banner--error">
            <span>
              <strong>sing-box не установлен</strong> — без него VPN не работает. В PowerShell:{" "}
              <code>.\scripts\install-singbox.ps1</code>, затем перезапустите приложение{" "}
              <strong>от администратора</strong>.
            </span>
          </div>
        )}

        {status.state === "error" && status.message && (
          <div className="banner banner--error">
            <span>{status.message}</span>
            {status.message.toLowerCase().includes("object already exists") && (
              <button
                type="button"
                className="btn btn--ghost btn--sm"
                disabled={busy}
                onClick={() =>
                  void resetTunAdapter()
                    .then(() => {
                      toast.success("TUN сброшен — попробуйте подключиться снова");
                      void refreshStatus();
                    })
                    .catch((e) => toast.error(toMessage(e)))
                }
              >
                Сбросить TUN
              </button>
            )}
          </div>
        )}

        {error && (
          <div className="banner banner--error">
            <span>{error}</span>
            <button type="button" className="btn btn--ghost btn--sm" onClick={() => void refresh()}>
              Повторить
            </button>
          </div>
        )}

        <ProfileList
          profiles={profiles}
          loading={loading}
          status={status}
          displayStatus={displayStatus}
          busy={busy}
          onConnect={(p) => void handleConnect(p)}
          onDisconnect={() => void handleDisconnect()}
          onRename={handleRename}
          onDelete={(id) => void handleDelete(id)}
          onImport={() => setImportOpen(true)}
        />

        <AppFooterShowcase status={status} />
        </main>

        <SidebarRight profiles={profiles} status={displayStatus} />
      </div>

      <BottomBar
        status={displayStatus}
        busy={busy}
        onImport={() => setImportOpen(true)}
        onDisconnect={() => void handleDisconnect()}
      />

      {importOpen && (
        <ImportKeyModal
          onClose={() => setImportOpen(false)}
          onImportKey={importKey}
          onImportSubscription={importSub}
          onImported={(n) => toast.success(n === 1 ? "Ключ добавлен" : `Добавлено профилей: ${n}`)}
        />
      )}

      {settingsOpen && (
        <SettingsModal
          settings={settings}
          connected={status.state === "connected"}
          onSettingsChange={setSettings}
          onClose={() => setSettingsOpen(false)}
        />
      )}
    </div>
  );
}
