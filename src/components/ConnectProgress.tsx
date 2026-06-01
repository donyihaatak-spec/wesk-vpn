interface ConnectProgressProps {
  mode: "connecting" | "disconnecting";
}

const STEPS_CONNECT = ["Конфигурация", "sing-box", "TUN-адаптер"] as const;
const STEPS_DISCONNECT = ["Закрытие", "Очистка TUN", "Готово"] as const;

export function ConnectProgress({ mode }: ConnectProgressProps) {
  const steps = mode === "connecting" ? STEPS_CONNECT : STEPS_DISCONNECT;

  return (
    <div className={`connect-progress connect-progress--${mode}`} aria-hidden>
      <div className="connect-progress__track">
        <span className="connect-progress__fill" />
      </div>
      <ul className="connect-progress__steps">
        {steps.map((label, i) => (
          <li key={label} className={`connect-progress__step connect-progress__step--${i}`}>
            <span className="connect-progress__dot" />
            <span className="connect-progress__label">{label}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
