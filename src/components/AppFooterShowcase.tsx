import { BRAND } from "../lib/brand";
import type { ProxyStatus } from "../lib/tauri";

interface AppFooterShowcaseProps {
  status: ProxyStatus;
}

export function AppFooterShowcase({ status }: AppFooterShowcaseProps) {
  const connected = status.state === "connected";
  const busy = status.state === "connecting" || status.state === "disconnecting";

  return (
    <footer className={`app-footer app-footer--${status.state}`} aria-hidden>
      <div className="app-footer__stage">
        <div className="app-footer__rings">
          <span className="app-footer__ring app-footer__ring--1" />
          <span className="app-footer__ring app-footer__ring--2" />
          <span className="app-footer__ring app-footer__ring--3" />
        </div>

        <div className={`app-footer__orb${busy ? " app-footer__orb--busy" : ""}${connected ? " app-footer__orb--live" : ""}`}>
          <svg className="app-footer__logo" viewBox="0 0 48 48" fill="none" aria-hidden>
            <defs>
              <linearGradient id="footer-grad" x1="6" y1="42" x2="42" y2="6">
                <stop offset="0%" stopColor="#ff6b4a" />
                <stop offset="50%" stopColor="#f43f8f" />
                <stop offset="100%" stopColor="#8b5cf6" />
              </linearGradient>
            </defs>
            <path
              d="M8 14 L14 34 L24 18 L34 34 L40 14"
              stroke="url(#footer-grad)"
              strokeWidth="3.2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <circle className="app-footer__logo-dot" cx="24" cy="38" r="2.5" fill="url(#footer-grad)" />
          </svg>
          <span className="app-footer__scan" />
        </div>

        {[0, 1, 2, 3, 4, 5].map((i) => (
          <span key={i} className={`app-footer__particle app-footer__particle--${i}`} />
        ))}
      </div>

      <p className="app-footer__brand">{BRAND.name}</p>
      <p className="app-footer__tag">{BRAND.protocolLine}</p>

      <div className="app-footer__chips">
        <span className="app-footer__chip">VLESS</span>
        <span className="app-footer__chip">VMess</span>
        <span className="app-footer__chip">Trojan</span>
        <span className="app-footer__chip">sing-box</span>
        <span className="app-footer__chip">Split tunnel</span>
      </div>

      <div className="app-footer__wave" aria-hidden>
        <svg viewBox="0 0 1200 80" preserveAspectRatio="none">
          <path
            className="app-footer__wave-path app-footer__wave-path--1"
            d="M0,40 Q150,10 300,40 T600,40 T900,40 T1200,40 L1200,80 L0,80 Z"
          />
          <path
            className="app-footer__wave-path app-footer__wave-path--2"
            d="M0,50 Q200,75 400,50 T800,50 T1200,50 L1200,80 L0,80 Z"
          />
        </svg>
      </div>
    </footer>
  );
}
