// API Types for Fylge

export type UUID = string;
export type EpochMs = number; // Milliseconds since Unix epoch
export type IconId = string;

// A marker in the log
export interface Marker {
  id: number;
  uuid: UUID;
  ts_epoch_ms: EpochMs;
  lat: number;
  lon: number;
  icon_id: IconId;
  label: string | null;
}

// Request to create a new marker
export interface CreateMarkerRequest {
  uuid: UUID;
  lat: number;
  lon: number;
  icon_id: IconId;
  label?: string;
}

// Response for creating a marker
export interface CreateMarkerResponse {
  status: "created" | "exists";
  marker: Marker;
}

// Response for GET /api/markers
export interface GetMarkersResponse {
  window_hours: number;
  server_time_ms: EpochMs;
  max_id: number;
  markers: Marker[];
}

// Response for GET /api/markers_at
export interface GetMarkersAtResponse {
  at_epoch_ms: EpochMs;
  window_hours: number;
  markers: Marker[];
}

// Response for GET /api/log
export interface GetLogResponse {
  after_id: number;
  limit: number;
  server_time_ms: EpochMs;
  max_id: number;
  has_more: boolean;
  entries: Marker[];
}

// Icon metadata
export interface Icon {
  id: IconId;
  name: string;
  url: string;
}

// Response for GET /api/icons
export interface GetIconsResponse {
  icons: Icon[];
}

// Application state
export interface AppState {
  lastId: number;
  markersByUuid: Map<UUID, Marker>;
  selectedIconId: IconId | null;
  icons: Icon[];
}
