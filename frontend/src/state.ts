// Application state management

import type { AppState, Marker, Icon, UUID } from './types';

const STORAGE_KEY = 'fylge_state';

export function createState(): AppState {
  return {
    lastId: loadLastId(),
    markersByUuid: new Map(),
    selectedIconId: null,
    icons: [],
  };
}

function loadLastId(): number {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      return data.lastId ?? 0;
    }
  } catch {
    // Ignore parse errors
  }
  return 0;
}

export function saveLastId(lastId: number): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ lastId }));
  } catch {
    // Ignore storage errors
  }
}

export function addMarker(state: AppState, marker: Marker): void {
  state.markersByUuid.set(marker.uuid, marker);
}

export function removeExpiredMarkers(state: AppState): UUID[] {
  const now = new Date();
  const cutoff = new Date(now.getTime() - 24 * 60 * 60 * 1000);
  const removed: UUID[] = [];

  for (const [uuid, marker] of state.markersByUuid) {
    const markerTime = new Date(marker.ts);
    if (markerTime < cutoff) {
      state.markersByUuid.delete(uuid);
      removed.push(uuid);
    }
  }

  return removed;
}

export function setIcons(state: AppState, icons: Icon[]): void {
  state.icons = icons;
  // Select first icon by default if none selected
  if (!state.selectedIconId && icons.length > 0) {
    state.selectedIconId = icons[0].id;
  }
}

export function selectIcon(state: AppState, iconId: string): void {
  state.selectedIconId = iconId;
}

export function getMarkersArray(state: AppState): Marker[] {
  return Array.from(state.markersByUuid.values());
}
