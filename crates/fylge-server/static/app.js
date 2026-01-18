// Fylge Globe Application

let globe;
let selectedIconId = null;
let markers = [];

// Initialize globe
function initGlobe() {
  const container = document.getElementById("globe-container");

  globe = Globe()
    .globeImageUrl("//unpkg.com/three-globe/example/img/earth-blue-marble.jpg")
    .bumpImageUrl("//unpkg.com/three-globe/example/img/earth-topology.png")
    .backgroundImageUrl("//unpkg.com/three-globe/example/img/night-sky.png")
    .showAtmosphere(true)
    .atmosphereColor("#3a228a")
    .atmosphereAltitude(0.25)
    .htmlElementsData(markers)
    .htmlElement((d) => {
      const el = document.createElement("div");
      el.style.pointerEvents = "none";

      const img = document.createElement("img");
      img.src = `/static/icons/${d.icon_id}.svg`;
      img.style.width = "24px";
      img.style.height = "24px";
      img.style.filter = "drop-shadow(0 0 3px rgba(0,0,0,0.5))";
      el.appendChild(img);

      if (d.label) {
        const label = document.createElement("div");
        label.className = "marker-label";
        label.textContent = d.label;
        el.appendChild(label);
      }

      return el;
    })
    .htmlLat((d) => d.lat)
    .htmlLng((d) => d.lon)
    .htmlAltitude(0.01)
    .onGlobeClick(({ lat, lng }) => {
      if (selectedIconId) {
        createMarker(lat, lng, selectedIconId);
      }
    })(container);

  // Set initial view
  globe.pointOfView({ lat: 20, lng: 0, altitude: 2.5 });

  // Load initial markers
  loadMarkers();
}

// Load markers from API
async function loadMarkers() {
  try {
    const response = await fetch("/api/markers");
    markers = await response.json();
    globe.htmlElementsData(markers);
    updateMarkerCount(markers.length);
  } catch (error) {
    console.error("Failed to load markers:", error);
  }
}

// Create a new marker
async function createMarker(lat, lon, iconId) {
  const params = new URLSearchParams();
  params.append("lat", lat.toFixed(6));
  params.append("lon", lon.toFixed(6));
  params.append("icon_id", iconId);

  try {
    const response = await fetch("/markers", {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: params,
    });

    const html = await response.text();
    document.getElementById("messages").innerHTML = html;

    if (response.ok) {
      loadMarkers();
      setTimeout(() => {
        document.getElementById("messages").innerHTML = "";
      }, 3000);
    }
  } catch (error) {
    console.error("Failed to create marker:", error);
    document.getElementById("messages").innerHTML =
      '<div class="error">Failed to create marker</div>';
  }
}

// Update marker count display
function updateMarkerCount(count) {
  const countEl = document.getElementById("marker-count");
  if (countEl) {
    countEl.textContent = count;
  }
}

// Icon selection
function setupIconPalette() {
  const buttons = document.querySelectorAll(".icon-btn");

  buttons.forEach((btn) => {
    btn.addEventListener("click", () => {
      // Deselect all
      buttons.forEach((b) => b.classList.remove("selected"));

      // Select this one
      btn.classList.add("selected");
      selectedIconId = btn.dataset.iconId;
    });
  });
}

// Listen for htmx events
document.body.addEventListener("markersChanged", () => {
  loadMarkers();
});

// Initialize on load
document.addEventListener("DOMContentLoaded", () => {
  setupIconPalette();
  initGlobe();
});
