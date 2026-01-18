// Fylge Globe Application

let globe;
let selectedIcon = null;
let markers = new Map(); // uuid -> marker data

// Initialize the application
document.addEventListener("DOMContentLoaded", async () => {
  await loadIcons();
  initGlobe();
  await loadMarkers();
});

// Load available icons
async function loadIcons() {
  try {
    const response = await fetch("/api/icons");
    const icons = await response.json();

    const palette = document.getElementById("icon-palette");
    palette.innerHTML = "";

    icons.forEach((icon) => {
      const btn = document.createElement("button");
      btn.className = "icon-btn";
      btn.dataset.iconId = icon.id;
      btn.innerHTML = `
        <img src="${icon.url}" alt="${icon.name}">
        <span>${icon.name}</span>
      `;
      btn.addEventListener("click", () => selectIcon(icon.id));
      palette.appendChild(btn);
    });

    // Select first icon by default
    if (icons.length > 0) {
      selectIcon(icons[0].id);
    }
  } catch (error) {
    console.error("Failed to load icons:", error);
  }
}

// Select an icon
function selectIcon(iconId) {
  selectedIcon = iconId;

  document.querySelectorAll(".icon-btn").forEach((btn) => {
    btn.classList.toggle("selected", btn.dataset.iconId === iconId);
  });
}

// Initialize globe.gl
function initGlobe() {
  const container = document.getElementById("globe");

  globe = Globe()(container)
    .globeImageUrl(
      "https://unpkg.com/three-globe/example/img/earth-blue-marble.jpg"
    )
    .bumpImageUrl(
      "https://unpkg.com/three-globe/example/img/earth-topology.png"
    )
    .backgroundImageUrl(
      "https://unpkg.com/three-globe/example/img/night-sky.png"
    )
    .htmlElementsData([])
    .htmlElement((d) => {
      const el = document.createElement("div");
      el.innerHTML = `<img src="/static/icons/${d.icon_id}.svg" style="width: 24px; height: 24px; cursor: pointer;">`;
      el.style.pointerEvents = "auto";
      el.onclick = () => showMarkerInfo(d);
      return el;
    })
    .onGlobeClick(({ lat, lng }) => {
      if (selectedIcon) {
        createMarker(lat, lng, selectedIcon);
      }
    });

  // Handle window resize
  window.addEventListener("resize", () => {
    globe.width(container.clientWidth);
    globe.height(container.clientHeight);
  });
}

// Load markers from server
async function loadMarkers() {
  try {
    const response = await fetch("/api/markers");
    const data = await response.json();

    markers.clear();
    data.forEach((m) => {
      markers.set(m.uuid, m);
    });

    updateGlobe();
    updateMarkerList();
  } catch (error) {
    console.error("Failed to load markers:", error);
  }
}

// Create a new marker
async function createMarker(lat, lon, iconId) {
  try {
    const response = await fetch("/markers", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        lat: lat,
        lon: lon,
        icon_id: iconId,
      }),
    });

    if (response.ok) {
      await loadMarkers();
    } else {
      const error = await response.text();
      console.error("Failed to create marker:", error);
    }
  } catch (error) {
    console.error("Failed to create marker:", error);
  }
}

// Delete a marker
async function deleteMarker(uuid) {
  try {
    const response = await fetch(`/markers/${uuid}`, {
      method: "DELETE",
    });

    if (response.ok) {
      await loadMarkers();
    } else {
      const error = await response.text();
      console.error("Failed to delete marker:", error);
    }
  } catch (error) {
    console.error("Failed to delete marker:", error);
  }
}

// Update globe with current markers
function updateGlobe() {
  const data = Array.from(markers.values());
  globe.htmlElementsData(data);
}

// Update marker list in sidebar
function updateMarkerList() {
  const list = document.getElementById("marker-list");
  list.innerHTML = "";

  markers.forEach((marker) => {
    const item = document.createElement("div");
    item.className = "marker-item";
    item.innerHTML = `
      <div class="marker-info">
        <img src="/static/icons/${marker.icon_id}.svg" alt="${marker.icon_id}">
        <span class="marker-coords">${marker.lat.toFixed(2)}, ${marker.lon.toFixed(2)}</span>
      </div>
      <button class="delete-btn" onclick="deleteMarker('${marker.uuid}')">Delete</button>
    `;
    list.appendChild(item);
  });
}

// Show marker info (could be expanded)
function showMarkerInfo(marker) {
  console.log("Marker clicked:", marker);
}
