// API client for Fylge backend

import type {
  CreateMarkerRequest,
  CreateMarkerResponse,
  GetMarkersResponse,
  GetLogResponse,
  GetIconsResponse,
} from './types';

const API_BASE = '';

export async function createMarker(req: CreateMarkerRequest): Promise<CreateMarkerResponse> {
  const response = await fetch(`${API_BASE}/markers`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Failed to create marker: ${response.status} ${text}`);
  }

  return response.json();
}

export async function getMarkers(): Promise<GetMarkersResponse> {
  const response = await fetch(`${API_BASE}/api/markers`);

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Failed to get markers: ${response.status} ${text}`);
  }

  return response.json();
}

export async function getLog(afterId: number, limit: number = 100): Promise<GetLogResponse> {
  const params = new URLSearchParams({
    after_id: afterId.toString(),
    limit: limit.toString(),
  });

  const response = await fetch(`${API_BASE}/api/log?${params}`);

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Failed to get log: ${response.status} ${text}`);
  }

  return response.json();
}

export async function getIcons(): Promise<GetIconsResponse> {
  const response = await fetch(`${API_BASE}/api/icons`);

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Failed to get icons: ${response.status} ${text}`);
  }

  return response.json();
}
