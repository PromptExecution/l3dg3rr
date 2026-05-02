import { taskbarBus } from '../taskbar/bus.js';

const MODEL_ID = 'Xenova/Phi-3.5-mini-instruct'; // ONNX int4 quantized
const DOWNLOAD_ID = 'llm-model';

// transformers.js untyped
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let _pipeline: any = null;

export async function getOrLoadPipeline(): Promise<unknown> {
  if (_pipeline) return _pipeline;

  taskbarBus.publish({
    type: 'download:start',
    item: {
      id: DOWNLOAD_ID,
      label: 'Phi-3.5-mini (int4)',
      progress: 0,
      status: 'pending',
    },
  });

  // transformers.js untyped
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const { pipeline, env } = await import('@huggingface/transformers') as any;

  // Use OPFS cache so second load is instant
  env.useBrowserCache  = true;
  env.allowLocalModels = false;

  // transformers.js untyped
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  _pipeline = await pipeline('text-generation', MODEL_ID, {
    progress_callback: (info: any) => { // transformers.js untyped
      if (info.status === 'downloading') {
        const total    = typeof info.total === 'number' ? info.total : 0;
        const loaded   = typeof info.loaded === 'number' ? info.loaded : 0;
        const progress = total > 0 ? loaded / total : 0;
        const detail   = total > 0
          ? `${(loaded / 1e6).toFixed(0)} / ${(total / 1e6).toFixed(0)} MB`
          : undefined;
        taskbarBus.publish({ type: 'download:progress', id: DOWNLOAD_ID, progress, detail });
      } else if (info.status === 'done') {
        taskbarBus.publish({ type: 'download:complete', id: DOWNLOAD_ID });
        taskbarBus.publish({
          type: 'toast',
          toast: {
            id: crypto.randomUUID(),
            message: 'Phi-3.5-mini ready',
            level: 'success',
            ttl: 3000,
          },
        });
      }
    },
  });

  return _pipeline;
}
