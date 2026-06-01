// Универсальный значок-статус (pill) с цветовым тоном и точкой-индикатором.
// Используется и для статуса VPN, и для валидности конфигурации.

import type { ReactNode } from "react";

export type BadgeTone = "ok" | "warn" | "danger" | "info" | "muted";

interface StatusBadgeProps {
  tone: BadgeTone;
  children: ReactNode;
  /** Пульсирующая точка — для переходных состояний (подключение/отключение). */
  pulse?: boolean;
}

export function StatusBadge({ tone, children, pulse = false }: StatusBadgeProps) {
  return (
    <span className={`badge badge--${tone}${pulse ? " badge--pulse" : ""}`}>
      <span className="badge__dot" aria-hidden />
      <span className="badge__label">{children}</span>
    </span>
  );
}
