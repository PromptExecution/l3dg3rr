import type { TaskbarEvent } from './types.js';

type Handler = (event: TaskbarEvent) => void;

class EventBus {
  private handlers: Handler[] = [];

  subscribe(fn: Handler): () => void {
    this.handlers.push(fn);
    return () => {
      this.handlers = this.handlers.filter(h => h !== fn);
    };
  }

  publish(event: TaskbarEvent): void {
    for (const h of this.handlers) {
      try {
        h(event);
      } catch (err) {
        console.error('[taskbarBus] handler error', err);
      }
    }
  }
}

export const taskbarBus = new EventBus();
