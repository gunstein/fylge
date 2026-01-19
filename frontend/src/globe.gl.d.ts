// Type declarations for globe.gl

declare module 'globe.gl' {
  interface GlobeInstance {
    (element: HTMLElement): GlobeInstance;

    // Globe appearance
    globeImageUrl(url: string): GlobeInstance;
    bumpImageUrl(url: string): GlobeInstance;
    backgroundImageUrl(url: string): GlobeInstance;

    // HTML elements layer
    htmlElementsData(data: object[]): GlobeInstance;
    htmlElement(fn: (d: object) => HTMLElement): GlobeInstance;
    htmlLat(fn: (d: object) => number): GlobeInstance;
    htmlLng(fn: (d: object) => number): GlobeInstance;
    htmlAltitude(alt: number): GlobeInstance;

    // Events
    onGlobeClick(fn: (coords: { lat: number; lng: number }) => void): GlobeInstance;

    // Sizing
    width(w: number): GlobeInstance;
    height(h: number): GlobeInstance;
  }

  function Globe(): GlobeInstance;
  export default Globe;
}
