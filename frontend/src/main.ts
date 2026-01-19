// Fylge - Globe marker application

import Globe from "globe.gl";
import type { Marker } from "./types";
import * as api from "./api";
import * as state from "./state";

// Generate UUID v4
function generateUUID(): string {
  return crypto.randomUUID();
}

// Application state
const appState = state.createState();

// Globe instance
let globe: ReturnType<typeof Globe> | null = null;

// Polling interval (ms)
const POLL_INTERVAL = 3000;

// Expiry check interval (ms)
const EXPIRY_CHECK_INTERVAL = 60000;

// Initialize the application
async function init(): Promise<void> {
  console.log("Initializing Fylge...");

  // Load icons
  try {
    const iconsResponse = await api.getIcons();
    state.setIcons(appState, iconsResponse.icons);
    renderIconPalette();
  } catch (e) {
    console.error("Failed to load icons:", e);
  }

  // Initialize globe
  initGlobe();

  // Load initial markers
  await loadInitialMarkers();

  // Start polling for new markers
  startPolling();

  // Start expiry checking
  startExpiryCheck();
}

function initGlobe(): void {
  const container = document.getElementById("globe-container");
  if (!container) {
    console.error("Globe container not found");
    return;
  }

  globe = Globe()(container)
    .globeImageUrl("//unpkg.com/three-globe/example/img/earth-blue-marble.jpg")
    .bumpImageUrl("//unpkg.com/three-globe/example/img/earth-topology.png")
    .backgroundImageUrl("//unpkg.com/three-globe/example/img/night-sky.png")
    .htmlElementsData([])
    .htmlElement((d: object) => {
      const marker = d as Marker;
      const icon = appState.icons.find((i) => i.id === marker.icon_id);

      const el = document.createElement("div");
      el.className = "marker";
      el.innerHTML = `
        <img src="${icon?.url ?? "/static/icons/marker.svg"}" alt="${marker.icon_id}" />
        ${marker.label ? `<span class="label">${marker.label}</span>` : ""}
      `;
      el.style.pointerEvents = "auto";
      el.style.cursor = "pointer";
      el.title = marker.label ?? marker.uuid;

      return el;
    })
    .htmlLat((d: object) => (d as Marker).lat)
    .htmlLng((d: object) => (d as Marker).lon)
    .htmlAltitude(0.01)
    .onGlobeClick(handleGlobeClick);

  // Resize handler
  window.addEventListener("resize", () => {
    if (globe && container) {
      globe.width(container.clientWidth);
      globe.height(container.clientHeight);
    }
  });
}

async function handleGlobeClick(coords: {
  lat: number;
  lng: number;
}): Promise<void> {
  if (!appState.selectedIconId) {
    console.log("No icon selected");
    return;
  }

  const uuid = generateUUID();
  const label = prompt("Label (optional):") ?? undefined;

  try {
    const response = await api.createMarker({
      uuid,
      lat: coords.lat,
      lon: coords.lng,
      icon_id: appState.selectedIconId,
      label: label || undefined,
    });

    console.log(`Marker ${response.status}:`, response.marker);

    // Add to local state
    state.addMarker(appState, response.marker);
    updateGlobeMarkers();

    // Update lastId if this is newer
    if (response.marker.id > appState.lastId) {
      appState.lastId = response.marker.id;
      state.saveLastId(appState.lastId);
    }
  } catch (e) {
    console.error("Failed to create marker:", e);
    alert("Failed to create marker. Please try again.");
  }
}

async function loadInitialMarkers(): Promise<void> {
  try {
    const response = await api.getMarkers();
    console.log(`Loaded ${response.markers.length} markers (last 24h)`);

    for (const marker of response.markers) {
      state.addMarker(appState, marker);
    }

    appState.lastId = response.max_id;
    state.saveLastId(appState.lastId);

    updateGlobeMarkers();
  } catch (e) {
    console.error("Failed to load initial markers:", e);
  }
}

function startPolling(): void {
  setInterval(async () => {
    try {
      const response = await api.getLog(appState.lastId);

      if (response.entries.length > 0) {
        console.log(`Poll: ${response.entries.length} new entries`);

        for (const marker of response.entries) {
          state.addMarker(appState, marker);
        }

        appState.lastId = response.max_id;
        state.saveLastId(appState.lastId);

        updateGlobeMarkers();
      }
    } catch (e) {
      console.error("Polling error:", e);
    }
  }, POLL_INTERVAL);
}

function startExpiryCheck(): void {
  setInterval(() => {
    const removed = state.removeExpiredMarkers(appState);
    if (removed.length > 0) {
      console.log(`Removed ${removed.length} expired markers`);
      updateGlobeMarkers();
    }
  }, EXPIRY_CHECK_INTERVAL);
}

function updateGlobeMarkers(): void {
  if (!globe) return;

  const markers = state.getMarkersArray(appState);
  globe.htmlElementsData(markers);
}

function renderIconPalette(): void {
  const palette = document.getElementById("icon-palette");
  if (!palette) return;

  palette.innerHTML = "";

  for (const icon of appState.icons) {
    const btn = document.createElement("button");
    btn.className =
      "icon-btn" + (icon.id === appState.selectedIconId ? " selected" : "");
    btn.title = icon.name;
    btn.innerHTML = `<img src="${icon.url}" alt="${icon.name}" />`;
    btn.addEventListener("click", () => {
      state.selectIcon(appState, icon.id);
      renderIconPalette(); // Re-render to update selection
    });
    palette.appendChild(btn);
  }
}

// Start the app
init().catch(console.error);
