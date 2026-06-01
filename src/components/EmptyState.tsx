import { BRAND } from "../lib/brand";
import { Logo } from "./Logo";

interface EmptyStateProps {
  onImport: () => void;
}

export function EmptyState({ onImport }: EmptyStateProps) {
  return (
    <div className="state-center empty">
      <Logo size={72} className="empty__logo" />
      <h2 className="empty__title">Первый сервер</h2>
      <p className="empty__text">
        Добавьте ключ <code>vless://</code> или <code>vmess://</code> — как в Happ. {BRAND.name}{" "}
        сам настроит туннель и split routing.
      </p>
      <button type="button" className="btn btn--primary btn--lg" onClick={onImport}>
        Добавить ключ
      </button>
    </div>
  );
}
