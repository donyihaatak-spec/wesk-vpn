interface ToggleProps {
  checked: boolean;
  disabled?: boolean;
  label: string;
  hint?: string;
  onChange: (checked: boolean) => void;
}

export function Toggle({ checked, disabled, label, hint, onChange }: ToggleProps) {
  return (
    <label className={`setting-row${disabled ? " setting-row--disabled" : ""}`}>
      <span className="setting-row__text">
        <span className="setting-row__label">{label}</span>
        {hint && <span className="setting-row__hint">{hint}</span>}
      </span>
      <input
        type="checkbox"
        className="toggle"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
    </label>
  );
}
