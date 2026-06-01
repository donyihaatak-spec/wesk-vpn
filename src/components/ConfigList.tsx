// Сетка карточек конфигураций с состояниями загрузки и пустоты.

import type { VpnConfigRecord, VpnStatus } from "../lib/tauri";
import { ConfigCard } from "./ConfigCard";
import { EmptyState } from "./EmptyState";
import { Spinner } from "./Spinner";

interface ConfigListProps {
  configs: VpnConfigRecord[];
  loading: boolean;
  status: VpnStatus;
  busy: boolean;
  onConnect: (config: VpnConfigRecord) => void;
  onDisconnect: () => void;
  onRename: (id: string, name: string) => Promise<void>;
  onDelete: (id: string) => void;
  onImport: () => void;
}

export function ConfigList({
  configs,
  loading,
  status,
  busy,
  onConnect,
  onDisconnect,
  onRename,
  onDelete,
  onImport,
}: ConfigListProps) {
  if (loading) {
    return (
      <div className="state-center">
        <Spinner />
        <p className="state-center__text">Загрузка конфигураций…</p>
      </div>
    );
  }

  if (configs.length === 0) {
    return <EmptyState onImport={onImport} />;
  }

  const connected = status.state === "connected";

  return (
    <div className="config-grid">
      {configs.map((config) => (
        <ConfigCard
          key={config.id}
          config={config}
          isActive={status.activeConfigId === config.id}
          connected={connected}
          busy={busy}
          onConnect={onConnect}
          onDisconnect={onDisconnect}
          onRename={onRename}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
