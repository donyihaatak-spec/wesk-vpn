import { useEffect, useRef, useState } from "react";

import { STATE_FADE_MS } from "../lib/smoothTransition";
import type { ProxyStatus } from "../lib/tauri";

function isTransitioning(state: ProxyStatus["state"]): boolean {
  return state === "connecting" || state === "disconnecting";
}

function isStable(state: ProxyStatus["state"]): boolean {
  return state === "connected" || state === "disconnected" || state === "error";
}

/**
 * Сглаживает смену статуса: при выходе из connecting/disconnecting
 * сначала плавное затухание, потом обновление UI.
 */
export function useDisplayStatus(status: ProxyStatus): ProxyStatus {
  const [display, setDisplay] = useState(status);
  const displayRef = useRef(status);
  displayRef.current = display;

  useEffect(() => {
    const same =
      status.state === displayRef.current.state &&
      status.activeProfileId === displayRef.current.activeProfileId &&
      status.activeProfileName === displayRef.current.activeProfileName;

    if (same) return;

    const leavingTransition = isTransitioning(displayRef.current.state);
    const enteringStable = isStable(status.state);

    if (leavingTransition && enteringStable) {
      document.documentElement.classList.add("conn-fade");
      const id = window.setTimeout(() => {
        setDisplay(status);
        window.requestAnimationFrame(() => {
          document.documentElement.classList.remove("conn-fade");
        });
      }, STATE_FADE_MS);
      return () => {
        window.clearTimeout(id);
        document.documentElement.classList.remove("conn-fade");
      };
    }

    setDisplay(status);
  }, [status]);

  return display;
}
