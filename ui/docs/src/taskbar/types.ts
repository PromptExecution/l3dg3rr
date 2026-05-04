export interface DownloadItem {
  id: string;
  label: string;
  progress: number;   // 0.0 – 1.0
  status: 'pending' | 'active' | 'complete' | 'error';
  detail?: string;    // e.g. "142 MB / 900 MB"
}

export interface Toast {
  id: string;
  message: string;
  level: 'info' | 'success' | 'warn' | 'error';
  ttl: number;        // ms before auto-dismiss
}

// Stub for future vCard support
export interface VCard {
  id: string;
  name: string;
  subtitle?: string;
  avatar_url?: string;
  action_label?: string;
  on_action?: () => void;
}

export type TaskbarEvent =
  | { type: 'download:start';    item: DownloadItem }
  | { type: 'download:progress'; id: string; progress: number; detail?: string }
  | { type: 'download:complete'; id: string }
  | { type: 'download:error';    id: string; message: string }
  | { type: 'toast';             toast: Toast }
  | { type: 'vcard:show';        vcard: VCard }
  | { type: 'vcard:dismiss';     id: string };
