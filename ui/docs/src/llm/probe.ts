const PROBE_PORTS = [3737, 11434, 8080]; // ledgerr-mcp, ollama, generic

export interface LocalService {
  baseUrl: string;
  name: string;
}

export async function probeLocalService(): Promise<LocalService | null> {
  for (const port of PROBE_PORTS) {
    try {
      const url = `http://localhost:${port}`;
      const resp = await fetch(`${url}/health`, { signal: AbortSignal.timeout(500) });
      if (resp.ok) {
        const data: Record<string, unknown> = await resp.json().catch(() => ({}));
        const name = typeof data['name'] === 'string' ? data['name'] : `localhost:${port}`;
        return { baseUrl: url, name };
      }
    } catch {
      // next port
    }
  }
  return null;
}
