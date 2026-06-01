import { useState } from "react";

import { PROTOCOL_LABEL, type ProfileRecord } from "../lib/tauri";

interface ProfileCardProps {
  profile: ProfileRecord;
  isActive: boolean;
  connected: boolean;
  transitioning: boolean;
  disconnecting: boolean;
  busy: boolean;
  onConnect: (p: ProfileRecord) => void;
  onDisconnect: () => void;
  onRename: (id: string, name: string) => Promise<void>;
  onDelete: (id: string) => void;
}

function protocolShort(protocol: ProfileRecord["protocol"]): string {
  const label = PROTOCOL_LABEL[protocol] ?? protocol;
  return label.slice(0, 4).toUpperCase();
}

export function ProfileCard({
  profile,
  isActive,
  connected,
  transitioning,
  disconnecting,
  busy,
  onConnect,
  onDisconnect,
  onRename,
  onDelete,
}: ProfileCardProps) {
  const [editing, setEditing] = useState(false);
  const [draftName, setDraftName] = useState(profile.name);
  const [saving, setSaving] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const activeConnected = isActive && connected;

  const submitRename = async (): Promise<void> => {
    const name = draftName.trim();
    if (!name || name === profile.name) {
      setEditing(false);
      return;
    }
    setSaving(true);
    try {
      await onRename(profile.id, name);
      setEditing(false);
    } finally {
      setSaving(false);
    }
  };

  return (
    <article
      className={`card${isActive ? " card--active" : ""}${activeConnected ? " card--live" : ""}${transitioning ? " card--connecting" : ""}`}
    >
      {isActive && (
        <div
          className={`card__connect-overlay${transitioning ? " card__connect-overlay--on" : ""}`}
          aria-live="polite"
        >
          <span className="card__connect-spinner" />
          <span className="card__connect-text">
            {disconnecting ? "Отключение…" : "Подключение…"}
          </span>
          <span className="card__connect-shimmer" />
        </div>
      )}
      <div className="card__top">
        <div className="card__protocol" aria-hidden>
          {protocolShort(profile.protocol)}
        </div>
        <div className="card__info">
          {editing ? (
            <input
              className="card__name-input"
              value={draftName}
              autoFocus
              disabled={saving}
              onChange={(e) => setDraftName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void submitRename();
                if (e.key === "Escape") setEditing(false);
              }}
            />
          ) : (
            <>
              <h3 className="card__name" title={profile.name}>
                {profile.name}
              </h3>
              <p className="card__server">{profile.server}</p>
            </>
          )}
        </div>
      </div>

      <ul className="card__meta">
        <li className="card__meta-item">
          <span className="card__meta-label">Порт</span>
          <span className="card__meta-value">{profile.port}</span>
        </li>
        <li className="card__meta-item">
          <span className="card__meta-label">Протокол</span>
          <span className="card__meta-value">{PROTOCOL_LABEL[profile.protocol]}</span>
        </li>
      </ul>

      {editing ? (
        <div className="card__actions">
          <div className="card__actions-row">
            <button
              type="button"
              className="btn btn--primary btn--sm"
              disabled={saving}
              onClick={() => void submitRename()}
            >
              {saving ? "…" : "Сохранить"}
            </button>
            <button type="button" className="btn btn--ghost btn--sm" onClick={() => setEditing(false)}>
              Отмена
            </button>
          </div>
        </div>
      ) : confirmDelete ? (
        <div className="card__actions">
          <span className="card__confirm">Удалить профиль?</span>
          <div className="card__actions-row">
            <button
              type="button"
              className="btn btn--danger btn--sm"
              disabled={busy}
              onClick={() => {
                setConfirmDelete(false);
                onDelete(profile.id);
              }}
            >
              Удалить
            </button>
            <button type="button" className="btn btn--ghost btn--sm" onClick={() => setConfirmDelete(false)}>
              Отмена
            </button>
          </div>
        </div>
      ) : (
        <>
          <div className="card__connect">
            {activeConnected ? (
              <button
                type="button"
                className="btn btn--ghost btn--lg card__connect"
                disabled={busy}
                onClick={onDisconnect}
              >
                Отключить
              </button>
            ) : (
              <button
                type="button"
                className="btn btn--primary btn--lg card__connect"
                disabled={busy}
                onClick={() => onConnect(profile)}
              >
                Подключить
              </button>
            )}
          </div>
          <div className="card__actions">
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy}
              onClick={() => {
                setDraftName(profile.name);
                setEditing(true);
              }}
            >
              Переименовать
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm card__delete"
              disabled={busy}
              onClick={() => setConfirmDelete(true)}
            >
              Удалить
            </button>
          </div>
        </>
      )}
    </article>
  );
}
