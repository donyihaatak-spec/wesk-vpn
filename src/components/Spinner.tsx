// Круговой индикатор загрузки.

interface SpinnerProps {
  label?: string;
}

export function Spinner({ label = "Загрузка" }: SpinnerProps) {
  return <span className="spinner" role="progressbar" aria-label={label} />;
}
