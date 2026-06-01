// Карточка одной конфигурации: метаданные + действия
// (подключить/отключить/переименовать/удалить).
// Переименование и подтверждение удаления реализованы инлайн — нативные
// window.prompt/confirm в Tauri-вебвью недоступны.

import { useState } from "react";

import type { VpnConfigRecord } from "../lib/tauri";
import { StatusBadge } from "./StatusBadge";

interface ConfigCardProps {
  config: VpnConfigRecord;
  /** Эта конфигурация выбрана как активная в менеджере VPN. */
  isActive: boolean;
  /** Глобальное состояние «подключено». */
  connected: boolean;
  /** Идёт операция connect/disconnect — действия блокируются. */
  busy: boolean;
  onConnect: (config: VpnConfigRecord) => void;
  onDisconnect: () => void;
  onRename: (id: string, name: string) => Promise<void>;
  onDelete: (id: string) => void;
}

export function ConfigCard({
  config,
  isActive,
  connected,
  busy,
  onConnect,
  onDisconnect,
  onRename,
  onDelete,
}: ConfigCardProps) {
  const [editing, setEditing] = useState<boolean>(false);
  const [draftName, setDraftName] = useState<string>(config.name);
  const [savingName, setSavingName] = useState<boolean>(false);
  const [confirmingDelete, setConfirmingDelete] = useState<boolean>(false);

  const activeConnected = isActive && connected;

  const startEditing = (): void => {
    setDraftName(config.name);
    setEditing(true);
  };

  const cancelEditing = (): void => {
    setEditing(false);
    setDraftName(config.name);
  };

  const submitRename = async (): Promise<void> => {
    const name = draftName.trim();
    if (!name || name === config.name) {
      cancelEditing();
      return;
    }
    setSavingName(true);
    try {
      await onRename(config.id, name);
      setEditing(false);
    } catch {
      // Ошибка уже показана уведомлением в App — остаёмся в режиме правки.
    } finally {
      setSavingName(false);
    }
  };

  return (
    <article className={`card${isActive ? " card--active" : ""}`}>
      <div className="card__head">
        {editing ? (
          <input
            className="card__name-input"
            value={draftName}
            autoFocus
            disabled={savingName}
            onChange={(e) => setDraftName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                void submitRename();
              } else if (e.key === "Escape") {
                cancelEditing();
              }
            }}
          />
        ) : (
          <h3 className="card__name" title={config.name}>
            {config.name}
          </h3>
        )}
        <StatusBadge tone={config.valid ? "ok" : "danger"}>
          {config.valid ? "Валиден" : "Повреждён"}
        </StatusBadge>
      </div>

      <dl className="card__meta">
        <div className="card__row">
          <dt>Endpoint</dt>
          <dd>{config.endpoint ?? "—"}</dd>
        </div>
        <div className="card__row">
          <dt>DNS</dt>
          <dd>{config.dns.length > 0 ? config.dns.join(", ") : "—"}</dd>
        </div>
        <div className="card__row">
          <dt>Адреса</dt>
          <dd>{config.addresses.length > 0 ? config.addresses.join(", ") : "—"}</dd>
        </div>
        <div className="card__row">
          <dt>Пиры</dt>
          <dd>{config.peerCount}</dd>
        </div>
      </dl>

      <div className="card__actions">
        {editing ? (
          <>
            <button
              type="button"
              className="btn btn--primary btn--sm"
              disabled={savingName}
              onClick={() => void submitRename()}
            >
              {savingName ? "Сохранение…" : "Сохранить"}
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={savingName}
              onClick={cancelEditing}
            >
              Отмена
            </button>
          </>
        ) : confirmingDelete ? (
          <>
            <span className="card__confirm">Удалить конфигурацию?</span>
            <button
              type="button"
              className="btn btn--danger btn--sm"
              disabled={busy}
              onClick={() => {
                setConfirmingDelete(false);
                onDelete(config.id);
              }}
            >
              Да, удалить
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              onClick={() => setConfirmingDelete(false)}
            >
              Отмена
            </button>
          </>
        ) : (
          <>
            {activeConnected ? (
              <button
                type="button"
                className="btn btn--ghost btn--sm"
                disabled={busy}
                onClick={onDisconnect}
              >
                Отключить
              </button>
            ) : (
              <button
                type="button"
                className="btn btn--primary btn--sm"
                disabled={busy || !config.valid}
                title={config.valid ? undefined : "Конфигурация повреждена"}
                onClick={() => onConnect(config)}
              >
                Подключить
              </button>
            )}
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              disabled={busy}
              onClick={startEditing}
            >
              Переименовать
            </button>
            <button
              type="button"
              className="btn btn--ghost btn--sm card__delete"
              disabled={busy}
              onClick={() => setConfirmingDelete(true)}
            >
              Удалить
            </button>
          </>
        )}
      </div>
    </article>
  );
}
