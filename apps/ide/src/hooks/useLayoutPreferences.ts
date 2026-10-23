import { useEffect, useState } from "react";

const STORAGE_KEY = "xezim-ide/layout-v1";
export interface LayoutPreferences {
  explorerCollapsed: boolean;
}

const defaultPreferences: LayoutPreferences = { explorerCollapsed: false };

function readPreferences(): LayoutPreferences {
  if (typeof window === "undefined") return defaultPreferences;
  try {
    const value: unknown = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? "null");
    if (typeof value === "object" && value !== null && "explorerCollapsed" in value && typeof value.explorerCollapsed === "boolean") {
      return { explorerCollapsed: value.explorerCollapsed };
    }
  } catch {
    // Ignore malformed or unavailable browser storage and use the defaults.
  }
  return defaultPreferences;
}

export function useLayoutPreferences(): [LayoutPreferences, (next: LayoutPreferences) => void] {
  const [preferences, setPreferences] = useState(readPreferences);

  useEffect(() => {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
    } catch {
      // Layout is a convenience preference; an unavailable store must not block editing.
    }
  }, [preferences]);

  return [preferences, setPreferences];
}
