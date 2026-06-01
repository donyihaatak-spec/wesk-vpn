import type { ProfileRecord, ProxyStatus } from "../lib/tauri";
import { EmptyState } from "./EmptyState";
import { ProfileCard } from "./ProfileCard";
import { Spinner } from "./Spinner";

interface ProfileListProps {
  profiles: ProfileRecord[];
  loading: boolean;
  status: ProxyStatus;
  displayStatus: ProxyStatus;
  busy: boolean;
  onConnect: (p: ProfileRecord) => void;
  onDisconnect: () => void;
  onRename: (id: string, name: string) => Promise<void>;
  onDelete: (id: string) => void;
  onImport: () => void;
}

export function ProfileList({
  profiles,
  loading,
  status,
  displayStatus,
  busy,
  onConnect,
  onDisconnect,
  onRename,
  onDelete,
  onImport,
}: ProfileListProps) {
  if (loading) {
    return (
      <div className="state-center">
        <Spinner />
        <p className="state-center__text">Загрузка серверов…</p>
      </div>
    );
  }

  if (profiles.length === 0) {
    return <EmptyState onImport={onImport} />;
  }

  const connected = status.state === "connected";
  const gridBusy = displayStatus.state === "connecting" || displayStatus.state === "disconnecting";

  return (
    <>
      <div className="section-head">
        <h2 className="section-head__title">Серверы</h2>
        <span className="section-head__count">{profiles.length}</span>
      </div>
      <div className={`config-grid${gridBusy ? " config-grid--busy" : ""}`}>
        {profiles.map((p) => {
          const isActive = displayStatus.activeProfileId === p.id;
          const transitioning =
            isActive &&
            (displayStatus.state === "connecting" || displayStatus.state === "disconnecting");

          return (
          <ProfileCard
            key={p.id}
            profile={p}
            isActive={isActive}
            connected={connected}
            transitioning={transitioning}
            disconnecting={displayStatus.state === "disconnecting" && isActive}
            busy={busy}
            onConnect={onConnect}
            onDisconnect={onDisconnect}
            onRename={onRename}
            onDelete={onDelete}
          />
          );
        })}
      </div>
    </>
  );
}
