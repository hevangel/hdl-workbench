import { useEffect, useState, type Dispatch, type SetStateAction } from "react";

const STORAGE_KEY = "xezim-ide/layout-v1";
export const DEFAULT_BOTTOM_PANEL_HEIGHT = 220;
export const MIN_BOTTOM_PANEL_HEIGHT = 80;
export const MAX_BOTTOM_PANEL_HEIGHT = 2000;

export interface LayoutPreferences {
  explorerCollapsed: boolean;
  bottomPanelHeight: number;
}

const defaultPreferences: LayoutPreferences = { explorerCollapsed: false, bottomPanelHeight: DEFAULT_BOTTOM_PANEL_HEIGHT };

export function clampBottomPanelHeight(height: number): number {
  if (!Number.isFinite(height)) return DEFAULT_BOTTOM_PANEL_HEIGHT;
  return Math.round(Math.min(MAX_BOTTOM_PANEL_HEIGHT, Math.max(MIN_BOTTOM_PANEL_HEIGHT, height)));
}

function readPreferences(): LayoutPreferences {
  if (typeof window === "undefined") return defaultPreferences;
  try {
    const value: unknown = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? "null");
    if (typeof value !== "object" || value === null) return defaultPreferences;
    // Read each field independently so an older or partly malformed record keeps its valid settings.
    const explorerCollapsed = "explorerCollapsed" in value && typeof value.explorerCollapsed === "boolean"
      ? value.explorerCollapsed
      : defaultPreferences.explorerCollapsed;
    const bottomPanelHeight = "bottomPanelHeight" in value && typeof value.bottomPanelHeight === "number"
      ? clampBottomPanelHeight(value.bottomPanelHeight)
      : defaultPreferences.bottomPanelHeight;
    return { explorerCollapsed, bottomPanelHeight };
  } catch {
    // Ignore malformed or unavailable browser storage and use the defaults.
  }
  return defaultPreferences;
}

export function useLayoutPreferences(): [LayoutPreferences, Dispatch<SetStateAction<LayoutPreferences>>] {
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
