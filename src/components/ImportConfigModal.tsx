// Модальное окно импорта: из файла (через системный диалог Tauri) или
// вставкой текста. Реальные вызовы бэкенда — без моков.

import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import { toMessage, type VpnConfigRecord } from "../lib/tauri";

type ImportTab = "file" | "text";

interface ImportConfigModalProps {
  onClose: () => void;
  onImportFromPath: (path: string) => Promise<VpnConfigRecord>;
  onImportFromText: (name: string, content: string) => Promise<VpnConfigRecord>;
  onImported: (config: VpnConfigRecord) => void;
}

export function ImportConfigModal({
  onClose,
  onImportFromPath,
  onImportFromText,
  onImported,
}: ImportConfigModalProps) {
  const [tab, setTab] = useState<ImportTab>("file");
  const [busy, setBusy] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState<string>("");
  const [content, setContent] = useState<string>("");

  const pickFile = async (): Promise<void> => {
    setError(null);
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: "Выберите файл WireGuard .conf",
        filters: [{ name: "WireGuard", extensions: ["conf"] }],
      });
      if (typeof selected !== "string") {
        return; // пользователь отменил выбор
      }
      setBusy(true);
      const record = await onImportFromPath(selected);
      onImported(record);
      onClose();
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const submitText = async (): Promise<void> => {
    setError(null);
    if (!content.trim()) {
      setError("Вставьте текст конфигурации");
      return;
    }
    setBusy(true);
    try {
      const record = await onImportFromText(name.trim() || "WireGuard", content);
      onImported(record);
      onClose();
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-label="Импорт конфигурации"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal__head">
          <h2 className="modal__title">Импорт конфигурации</h2>
          <button
            type="button"
            className="modal__close"
            onClick={onClose}
            aria-label="Закрыть"
          >
            ×
          </button>
        </div>

        <div className="tabs" role="tablist">
          <button
            type="button"
            role="tab"
            aria-selected={tab === "file"}
            className={`tab${tab === "file" ? " tab--active" : ""}`}
            onClick={() => setTab("file")}
          >
            Из файла
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={tab === "text"}
            className={`tab${tab === "text" ? " tab--active" : ""}`}
            onClick={() => setTab("text")}
          >
            Вставить текст
          </button>
        </div>

        {tab === "file" ? (
          <div className="modal__body">
            <p className="modal__hint">
              Выберите файл <code>.conf</code> WireGuard. Он будет проверен и
              сохранён локально.
            </p>
            <button
              type="button"
              className="btn btn--primary"
              onClick={() => void pickFile()}
              disabled={busy}
            >
              {busy ? "Импорт…" : "Выбрать файл…"}
            </button>
          </div>
        ) : (
          <div className="modal__body">
            <label className="field">
              <span className="field__label">Название</span>
              <input
                className="field__input"
                value={name}
                placeholder="Например, Home VPN"
                onChange={(e) => setName(e.target.value)}
                disabled={busy}
              />
            </label>
            <label className="field">
              <span className="field__label">Содержимое .conf</span>
              <textarea
                className="field__textarea"
                value={content}
                placeholder={"[Interface]\nPrivateKey = …\nAddress = 10.0.0.2/32\n\n[Peer]\nPublicKey = …\nEndpoint = host:51820\nAllowedIPs = 0.0.0.0/0"}
                rows={10}
                spellCheck={false}
                onChange={(e) => setContent(e.target.value)}
                disabled={busy}
              />
            </label>
            <button
              type="button"
              className="btn btn--primary"
              onClick={() => void submitText()}
              disabled={busy}
            >
              {busy ? "Импорт…" : "Импортировать"}
            </button>
          </div>
        )}

        {error && <div className="modal__error">{error}</div>}
      </div>
    </div>
  );
}
