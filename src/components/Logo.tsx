import { useId } from "react";

interface LogoProps {
  size?: number;
  className?: string;
}

/** Логотип Wesk — стилизованная буква W. */
export function Logo({ size = 40, className = "" }: LogoProps) {
  const gradId = useId();

  return (
    <svg
      className={`logo-mark ${className}`.trim()}
      width={size}
      height={size}
      viewBox="0 0 48 48"
      fill="none"
      aria-hidden
    >
      <defs>
        <linearGradient id={gradId} x1="6" y1="42" x2="42" y2="6">
          <stop offset="0%" stopColor="#ff6b4a" />
          <stop offset="50%" stopColor="#f43f8f" />
          <stop offset="100%" stopColor="#8b5cf6" />
        </linearGradient>
      </defs>
      <rect width="48" height="48" rx="14" fill={`url(#${gradId})`} opacity="0.2" />
      <path
        d="M8 14 L14 34 L24 18 L34 34 L40 14"
        stroke={`url(#${gradId})`}
        strokeWidth="3.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="24" cy="38" r="2.5" fill={`url(#${gradId})`} opacity="0.85" />
    </svg>
  );
}
