import { probeLocalService } from './probe.js';
import { getOrLoadPipeline } from './loader.js';
import { taskbarBus } from '../taskbar/bus.js';

export async function generate(
  prompt: string,
  onChunk: (text: string) => void,
): Promise<void> {
  const local = await probeLocalService();

  if (local) {
    // Stream from local service (OpenAI-compatible)
    const resp = await fetch(`${local.baseUrl}/v1/chat/completions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: 'phi4',
        messages: [{ role: 'user', content: prompt }],
        stream: true,
      }),
    });

    const reader = resp.body?.getReader();
    if (!reader) throw new Error('no stream body');

    const decoder = new TextDecoder();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      const chunk = decoder.decode(value);
      for (const line of chunk.split('\n')) {
        if (!line.startsWith('data: ')) continue;
        const data = line.slice(6).trim();
        if (data === '[DONE]') break;
        try {
          const parsed: unknown = JSON.parse(data);
          if (
            parsed !== null &&
            typeof parsed === 'object' &&
            'choices' in parsed
          ) {
            const choices = (parsed as Record<string, unknown>)['choices'];
            if (Array.isArray(choices) && choices.length > 0) {
              const delta = (choices[0] as Record<string, unknown>)['delta'];
              if (delta !== null && typeof delta === 'object' && 'content' in delta) {
                const content = (delta as Record<string, unknown>)['content'];
                if (typeof content === 'string' && content) {
                  onChunk(content);
                }
              }
            }
          }
        } catch {
          // skip malformed SSE line
        }
      }
    }
    return;
  }

  // In-browser fallback
  taskbarBus.publish({
    type: 'toast',
    toast: {
      id: crypto.randomUUID(),
      message: 'No local service — using in-browser Phi-3.5-mini',
      level: 'info',
      ttl: 4000,
    },
  });

  const pipe = await getOrLoadPipeline();
  // transformers.js untyped
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const result = await (pipe as any)(prompt, { max_new_tokens: 256 });
  // transformers.js untyped
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const text = (result as any[])[0]?.generated_text ?? '';
  if (text) onChunk(String(text));
}
