import type { DownloadItem, Toast, VCard } from './types.js';
import type { TaskbarManager } from './manager.js';

const TOAST_COLORS: Record<Toast['level'], string> = {
  info:    '#1d4ed8',
  success: '#15803d',
  warn:    '#b45309',
  error:   '#b91c1c',
};

function renderDownload(item: DownloadItem): HTMLElement {
  const pill = document.createElement('div');
  pill.className = `tb-download tb-download--${item.status}`;
  pill.dataset['id'] = item.id;

  const pct = Math.round(item.progress * 100);
  const icon = item.status === 'complete' ? '✓' :
               item.status === 'error'    ? '!' : '↓';

  pill.innerHTML = `
    <span class="tb-download__icon">${icon}</span>
    <span class="tb-download__label">${escapeHtml(item.label)}</span>
    <span class="tb-download__bar">
      <span class="tb-download__fill" style="width: ${pct}%"></span>
    </span>
    <span class="tb-download__pct">${pct}%</span>
    ${item.detail ? `<span class="tb-download__detail">${escapeHtml(item.detail)}</span>` : ''}
  `.trim();

  return pill;
}

function renderToast(toast: Toast, onDismiss: (id: string) => void): HTMLElement {
  const chip = document.createElement('div');
  chip.className = 'tb-toast tb-toast--slide-in';
  chip.dataset['id'] = toast.id;
  chip.style.setProperty('--toast-color', TOAST_COLORS[toast.level]);

  chip.innerHTML = `
    <span class="tb-toast__msg">${escapeHtml(toast.message)}</span>
    <button class="tb-toast__dismiss" title="Dismiss">×</button>
  `.trim();

  chip.querySelector('.tb-toast__dismiss')?.addEventListener('click', () => {
    onDismiss(toast.id);
  });

  return chip;
}

function renderVCard(vcard: VCard): HTMLElement {
  const card = document.createElement('div');
  card.className = 'tb-vcard';
  card.dataset['id'] = vcard.id;

  card.innerHTML = `
    ${vcard.avatar_url ? `<img class="tb-vcard__avatar" src="${escapeHtml(vcard.avatar_url)}" alt="" />` : ''}
    <div class="tb-vcard__body">
      <div class="tb-vcard__name">${escapeHtml(vcard.name)}</div>
      ${vcard.subtitle ? `<div class="tb-vcard__subtitle">${escapeHtml(vcard.subtitle)}</div>` : ''}
    </div>
    ${vcard.action_label ? `<button class="tb-vcard__action">${escapeHtml(vcard.action_label)}</button>` : ''}
  `.trim();

  if (vcard.on_action) {
    const btn = card.querySelector('.tb-vcard__action');
    btn?.addEventListener('click', () => vcard.on_action!());
  }

  return card;
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function renderTaskbar(container: HTMLElement, manager: TaskbarManager): void {
  // Preserve existing elements where possible (simple diffing by id)
  const existingIds = new Set<string>();
  for (const el of Array.from(container.children)) {
    const id = (el as HTMLElement).dataset['id'];
    if (id) existingIds.add(id);
  }

  // Build new contents
  const fragment = document.createDocumentFragment();

  // Downloads (left side)
  for (const item of manager.downloads.values()) {
    fragment.appendChild(renderDownload(item));
  }

  // Toasts (middle)
  for (const toast of manager.toasts.values()) {
    fragment.appendChild(renderToast(toast, id => manager.dismissToast(id)));
  }

  // Spacer
  const spacer = document.createElement('div');
  spacer.className = 'tb-spacer';
  fragment.appendChild(spacer);

  // VCards (right side)
  for (const vcard of manager.vcards.values()) {
    fragment.appendChild(renderVCard(vcard));
  }

  container.replaceChildren(fragment);
}
