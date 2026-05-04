import type { VizManifest, VizManifestEntry } from './types.js';
import { renderScene } from './canvas.js';
import { taskbarBus } from '../taskbar/bus.js';

const MANIFEST_DOWNLOAD_ID = 'viz-manifest';

export async function loadScene(
  svgEl: SVGSVGElement,
  onSelect: (entry: VizManifestEntry) => void,
): Promise<VizManifest> {
  taskbarBus.publish({
    type: 'download:start',
    item: {
      id: MANIFEST_DOWNLOAD_ID,
      label: 'iso-manifest',
      progress: 0,
      status: 'pending',
    },
  });

  const resp = await fetch('./viz-manifest.json');
  if (!resp.ok) {
    taskbarBus.publish({
      type: 'download:error',
      id: MANIFEST_DOWNLOAD_ID,
      message: `HTTP ${resp.status}`,
    });
    throw new Error(`Failed to load viz-manifest.json: ${resp.status}`);
  }

  const manifest: VizManifest = await resp.json();

  taskbarBus.publish({ type: 'download:complete', id: MANIFEST_DOWNLOAD_ID });

  renderScene(svgEl, manifest, onSelect);

  return manifest;
}
