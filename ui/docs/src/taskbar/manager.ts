import type { DownloadItem, Toast, VCard } from './types.js';
import { taskbarBus } from './bus.js';
import { renderTaskbar } from './render.js';

export class TaskbarManager {
  readonly downloads = new Map<string, DownloadItem>();
  readonly toasts    = new Map<string, Toast>();
  readonly vcards    = new Map<string, VCard>();

  private containerEl: HTMLElement;

  constructor(containerEl: HTMLElement) {
    this.containerEl = containerEl;
    taskbarBus.subscribe(event => {
      switch (event.type) {
        case 'download:start':
          this.downloads.set(event.item.id, { ...event.item, status: 'active' });
          break;
        case 'download:progress': {
          const item = this.downloads.get(event.id);
          if (item) {
            item.progress = event.progress;
            item.status   = 'active';
            if (event.detail !== undefined) item.detail = event.detail;
          }
          break;
        }
        case 'download:complete': {
          const item = this.downloads.get(event.id);
          if (item) {
            item.progress = 1;
            item.status   = 'complete';
            // Auto-remove after 3s
            setTimeout(() => {
              this.downloads.delete(event.id);
              this.render();
            }, 3000);
          }
          break;
        }
        case 'download:error': {
          const item = this.downloads.get(event.id);
          if (item) {
            item.status = 'error';
            item.detail = event.message;
          }
          break;
        }
        case 'toast': {
          this.toasts.set(event.toast.id, event.toast);
          if (event.toast.ttl > 0) {
            setTimeout(() => {
              this.toasts.delete(event.toast.id);
              this.render();
            }, event.toast.ttl);
          }
          break;
        }
        case 'vcard:show':
          this.vcards.set(event.vcard.id, event.vcard);
          break;
        case 'vcard:dismiss':
          this.vcards.delete(event.id);
          break;
      }
      this.render();
    });
  }

  dismissToast(id: string): void {
    this.toasts.delete(id);
    this.render();
  }

  render(): void {
    renderTaskbar(this.containerEl, this);
  }
}
