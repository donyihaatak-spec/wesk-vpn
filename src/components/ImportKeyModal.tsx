import { useState } from "react";

import { toMessage, type ProfileRecord } from "../lib/tauri";

type Tab = "key" | "subscription";

interface ImportKeyModalProps {
  onClose: () => void;
  onImportKey: (text: string) => Promise<ProfileRecord>;
  onImportSubscription: (url: string) => Promise<ProfileRecord[]>;
  onImported: (count: number) => void;
}

export function ImportKeyModal({
  onClose,
  onImportKey,
  onImportSubscription,
  onImported,
}: ImportKeyModalProps) {
  const [tab, setTab] = useState<Tab>("key");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [keyText, setKeyText] = useState("");
  const [subUrl, setSubUrl] = useState("");

  const submitKey = async (): Promise<void> => {
    setError(null);
    if (!keyText.trim()) {
      setError("Вставьте ключ (vless://, vmess://, trojan://, ss:// …)");
      return;
    }
    setBusy(true);
    try {
      await onImportKey(keyText.trim());
      onImported(1);
      onClose();
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setBusy(false);
    }
  };

  const submitSub = async (): Promise<void> => {
    setError(null);
    if (!subUrl.trim()) {
      setError("Введите URL подписки");
      return;
    }
    setBusy(true);
    try {
      const list = await onImportSubscription(subUrl.trim());
      onImported(list.length);
      onClose();
    } catch (e) {
      setError(toMessage(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" role="dialog" aria-modal="true" onClick={(e) => e.stopPropagation()}>
        <div className="modal__head">
          <h2 className="modal__title">Новый сервер</h2>
          <button type="button" className="modal__close" onClick={onClose} aria-label="Закрыть">
            ×
          </button>
        </div>

        <div className="tabs">
          <button type="button" className={`tab${tab === "key" ? " tab--active" : ""}`} onClick={() => setTab("key")}>
            Ключ
          </button>
          <button type="button" className={`tab${tab === "subscription" ? " tab--active" : ""}`} onClick={() => setTab("subscription")}>
            Подписка
          </button>
        </div>

        {tab === "key" ? (
          <div className="modal__body">
            <p className="modal__hint">
              Вставьте ключ из буфера обмена, как в Happ: <code>vless://</code>,{" "}
              <code>vmess://</code>, <code>trojan://</code>, <code>ss://</code> …
            </p>
            <label className="field">
              <span className="field__label">VPN-ключ</span>
              <textarea
                className="field__textarea"
                rows={6}
                spellCheck={false}
                placeholder="vless://uuid@host:443?security=reality&..."
                value={keyText}
                disabled={busy}
                onChange={(e) => setKeyText(e.target.value)}
              />
            </label>
            <button type="button" className="btn btn--primary" disabled={busy} onClick={() => void submitKey()}>
              {busy ? "Импорт…" : "Импортировать ключ"}
            </button>
          </div>
        ) : (
          <div className="modal__body">
            <p className="modal__hint">
              URL подписки от провайдера (список серверов загрузится автоматически).
            </p>
            <label className="field">
              <span className="field__label">URL подписки</span>
              <input
                className="field__input"
                type="url"
                placeholder="https://provider.example/sub/..."
                value={subUrl}
                disabled={busy}
                onChange={(e) => setSubUrl(e.target.value)}
              />
            </label>
            <button type="button" className="btn btn--primary" disabled={busy} onClick={() => void submitSub()}>
              {busy ? "Загрузка…" : "Загрузить подписку"}
            </button>
          </div>
        )}

        {error && <div className="modal__error">{error}</div>}
      </div>
    </div>
  );
}
