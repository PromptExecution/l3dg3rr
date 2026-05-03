// Tauri v2 IPC — __TAURI__ is injected after script parse, so access lazily
function invoke(cmd, args) { return window.__TAURI__.core.invoke(cmd, args); }
function listen(event, handler) { return window.__TAURI__.event.listen(event, handler); }

// ── State ─────────────────────────────────────────────────────────────────────
let activePanel = 0;
let activeLogPanel = 0;
let busy = false;

// ── DOM references ────────────────────────────────────────────────────────────
const sidebar        = document.getElementById('sidebar');
const collapseBtn    = document.getElementById('collapse-btn');
const navItems       = document.querySelectorAll('.nav-item[data-panel]');
const panels         = [0,1,2,3,4].map(i => document.getElementById(`panel-${i}`));

const versionText    = document.getElementById('version-text');
const statusBar      = document.getElementById('status-bar');
const appTitle       = document.getElementById('app-title'); // eslint-disable-line no-unused-vars

const modelBadge     = document.getElementById('model-badge');
const modelBadgeIcon = document.getElementById('model-badge-icon');
const modelBadgeText = document.getElementById('model-badge-text');

const pillPhi        = document.getElementById('pill-phi');
const pillFoundry    = document.getElementById('pill-foundry');
const pillCloud      = document.getElementById('pill-cloud');
const cloudHint      = document.getElementById('cloud-hint');

const transcript     = document.getElementById('transcript');
const draftInput     = document.getElementById('draft-input');
const sendBtn        = document.getElementById('send-btn');
const rhaiBtn        = document.getElementById('rhai-btn');

const rigLog         = document.getElementById('rig-log');
const reviewLog      = document.getElementById('review-log');
const docsRigLog     = document.getElementById('docs-rig-log');
const logTabs        = document.querySelectorAll('.log-tab');
const logPanels      = [0,1].map(i => document.getElementById(`log-panel-${i}`));

const inputEndpoint  = document.getElementById('input-endpoint');
const inputModel     = document.getElementById('input-model');
const inputApiKey    = document.getElementById('input-api-key');
const inputSysPrompt = document.getElementById('input-system-prompt');
const btnUsePhi      = document.getElementById('btn-use-phi');
const btnUseFoundry  = document.getElementById('btn-use-foundry');
const btnUseCloud    = document.getElementById('btn-use-cloud');
const btnSave        = document.getElementById('btn-save-settings');

const docsStatusText = document.getElementById('docs-status-text');
const btnOpenDocs    = document.getElementById('btn-open-docs');
const btnLoadRhai    = document.getElementById('btn-load-rhai-mutation');

const btnRefreshDash = document.getElementById('btn-refresh-dashboard');
const blockedValue   = document.getElementById('blocked-value');
const readyValue     = document.getElementById('ready-value');
const exportedValue  = document.getElementById('exported-value');
const issuesValue    = document.getElementById('issues-value');
const evLastAction   = document.getElementById('ev-last-action');
const evNextActions  = document.getElementById('ev-next-actions');
const evProviderStat = document.getElementById('ev-provider-status');

// ── Helpers ───────────────────────────────────────────────────────────────────

function setStatus(text) {
  statusBar.textContent = text;
}

function setBusy(val) {
  busy = val;
  sendBtn.disabled       = val;
  rhaiBtn.disabled       = val;
  draftInput.disabled    = val;
  pillPhi.disabled       = val;
  pillFoundry.disabled   = val;
  pillCloud.disabled     = val;
  btnUsePhi.disabled     = val;
  btnUseFoundry.disabled = val;
  btnUseCloud.disabled   = val;
  btnSave.disabled       = val;
  btnOpenDocs.disabled   = val;
  btnLoadRhai.disabled   = val;
  sendBtn.textContent    = val ? 'Sending…' : 'Send';
  btnSave.textContent    = val ? 'Working…' : 'Save Settings';
  inputEndpoint.disabled = val;
  inputModel.disabled    = val;
  inputApiKey.disabled   = val;
  inputSysPrompt.disabled = val;
}

function showPanel(index) {
  activePanel = index;
  panels.forEach((p, i) => p.classList.toggle('hidden', i !== index));
  navItems.forEach(item => {
    const panel = parseInt(item.dataset.panel, 10);
    item.classList.toggle('active', panel === index);
  });
}

function showLogPanel(index) {
  activeLogPanel = index;
  logPanels.forEach((p, i) => p.classList.toggle('hidden', i !== index));
  logTabs.forEach(tab => {
    tab.classList.toggle('active', parseInt(tab.dataset.log, 10) === index);
  });
}

function updateModelBadge() {
  const model = inputModel.value.trim();
  const apiKey = inputApiKey.value.trim();
  const isPhi  = model === 'phi-4-mini-reasoning' && apiKey === 'local-tool-tray';
  const isFoundry = model === 'phi-4-mini' && apiKey === 'local-foundry';
  modelBadgeIcon.textContent = isPhi ? '⚡' : (isFoundry ? 'WA' : '☁');
  modelBadgeText.textContent = model || 'No model — go to Settings';
  modelBadge.classList.toggle('phi',   isPhi);
  modelBadge.classList.toggle('foundry', isFoundry);
  modelBadge.classList.toggle('cloud', !isPhi && !isFoundry);
  pillPhi.classList.toggle('active',   isPhi);
  pillFoundry.classList.toggle('active', isFoundry);
  pillCloud.classList.toggle('active', !isPhi && !isFoundry && model !== '');
  cloudHint.classList.toggle('hidden', isPhi || isFoundry || model === '');
}

function applyChatSettings(payload) {
  inputEndpoint.value  = payload.endpoint_text  ?? '';
  inputModel.value     = payload.model_text      ?? '';
  inputApiKey.value    = payload.api_key_text    ?? '';
  inputSysPrompt.value = payload.system_prompt_text ?? '';
  updateModelBadge();
  if (payload.status_text) setStatus(payload.status_text);
}

function scrollToBottom(el) {
  el.scrollTop = el.scrollHeight;
}

// ── Sidebar collapse ──────────────────────────────────────────────────────────

collapseBtn.addEventListener('click', () => {
  sidebar.classList.toggle('collapsed');
  collapseBtn.querySelector('.mark').textContent =
    sidebar.classList.contains('collapsed') ? '>' : '<';
});

// ── Panel switching ───────────────────────────────────────────────────────────

navItems.forEach(item => {
  item.addEventListener('click', () => showPanel(parseInt(item.dataset.panel, 10)));
});

// ── Log tab switching ─────────────────────────────────────────────────────────

logTabs.forEach(tab => {
  tab.addEventListener('click', () => showLogPanel(parseInt(tab.dataset.log, 10)));
});

// ── Settings: Save ────────────────────────────────────────────────────────────

btnSave.addEventListener('click', async () => {
  try {
    const status = await invoke('save_settings', {
      endpoint:     inputEndpoint.value,
      model:        inputModel.value,
      apiKey:       inputApiKey.value,
      systemPrompt: inputSysPrompt.value,
    });
    setStatus(status);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

// ── Model pills ───────────────────────────────────────────────────────────────

pillPhi.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('use_internal_phi', {
      systemPrompt: inputSysPrompt.value,
    });
    applyChatSettings(payload);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

pillCloud.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('use_cloud_model', {
      systemPrompt: inputSysPrompt.value,
    });
    applyChatSettings(payload);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

pillFoundry.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('use_foundry_local', {
      systemPrompt: inputSysPrompt.value,
    });
    applyChatSettings(payload);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

btnUsePhi.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('use_internal_phi', {
      systemPrompt: inputSysPrompt.value,
    });
    applyChatSettings(payload);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

btnUseFoundry.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('use_foundry_local', {
      systemPrompt: inputSysPrompt.value,
    });
    applyChatSettings(payload);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

btnUseCloud.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('use_cloud_model', {
      systemPrompt: inputSysPrompt.value,
    });
    applyChatSettings(payload);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

// ── Chat: Send ────────────────────────────────────────────────────────────────

async function sendMessage() {
  if (busy) return;
  const draft = draftInput.value;
  if (!draft.trim()) {
    setStatus('Enter a message before sending.');
    return;
  }

  setBusy(true);

  try {
    const status = await invoke('send_message', {
      draft,
      endpoint:     inputEndpoint.value,
      model:        inputModel.value,
      apiKey:       inputApiKey.value,
      systemPrompt: inputSysPrompt.value,
    });
    setStatus(status);
  } catch (err) {
    setStatus(`Error: ${err}`);
    setBusy(false);
  }
  // busy=false is handled by the chat-update event with busy=false
}

sendBtn.addEventListener('click', sendMessage);

draftInput.addEventListener('keydown', e => {
  if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
    sendMessage();
  }
});

// ── Chat: Rhai Rule Prompt ────────────────────────────────────────────────────

rhaiBtn.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('load_rhai_rule_prompt', {
      currentModel:        inputModel.value,
      currentSystemPrompt: inputSysPrompt.value,
    });
    inputSysPrompt.value  = payload.system_prompt;
    if (payload.suggested_model) inputModel.value = payload.suggested_model;
    draftInput.value      = payload.draft_message;
    reviewLog.textContent = payload.review_log_text;
    scrollToBottom(reviewLog);
    setStatus(payload.status);
    updateModelBadge();
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

// ── Docs Playbook ─────────────────────────────────────────────────────────────

btnOpenDocs.addEventListener('click', async () => {
  if (busy) return;
  try {
    const status = await invoke('open_docs_playbook');
    setStatus(status);
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

btnLoadRhai.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('load_rhai_rule_prompt', {
      currentModel:        inputModel.value,
      currentSystemPrompt: inputSysPrompt.value,
    });
    inputSysPrompt.value  = payload.system_prompt;
    if (payload.suggested_model) inputModel.value = payload.suggested_model;
    draftInput.value      = payload.draft_message;
    reviewLog.textContent = payload.review_log_text;
    scrollToBottom(reviewLog);
    setStatus(payload.status);
    updateModelBadge();
    showPanel(0); // switch to Chat panel
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

// ── Dashboard: Refresh ────────────────────────────────────────────────────────

async function refreshDashboard() {
  try {
    const payload = await invoke('get_evidence_dashboard');
    const q = payload.today_queue;
    blockedValue.textContent  = q.blocked ?? '-';
    readyValue.textContent    = q.ready_to_review ?? '-';
    exportedValue.textContent = q.exported ?? '-';
    issuesValue.textContent   = q.with_validation_issues ?? '-';
    evLastAction.textContent  = q.last_action_summary ?? '';
    evNextActions.innerHTML   = (q.next_actions || []).map(a => `<li>${a}</li>`).join('');
    evProviderStat.innerHTML  = (q.providers || []).map(p => {
      const r = p.readiness || {};
      const status = r.status || r.kind || 'unknown';
      const icon = status === 'ready' ? '✓' : (status === 'diagnostic' ? '⚠' : '?');
      return `<div class="ev-provider-line">${icon} ${p.label}: ${status}</div>`;
    }).join('') || '<div class="ev-provider-line">No providers configured</div>';
  } catch (err) {
    evLastAction.textContent = `Error: ${err}`;
  }
}

// Refresh dashboard when its panel becomes visible
const origShowPanel = showPanel;
showPanel = function(index) {
  origShowPanel(index);
  if (index === 2) refreshDashboard();
};

btnRefreshDash.addEventListener('click', refreshDashboard);

// ── Initialise on load ────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', async () => {
  showPanel(0);
  showLogPanel(0);

  // Register chat-update listener now that __TAURI__ is available
  listen('chat-update', event => {
    const d = event.payload;
    transcript.textContent = d.transcript_text  ?? '';
    reviewLog.textContent  = d.review_log_text  ?? '';
    rigLog.textContent     = d.rig_log_text     ?? '';
    docsRigLog.textContent = d.rig_log_text     ?? '';

    if (typeof d.draft_message_text === 'string') {
      draftInput.value = d.draft_message_text;
    }
    if (d.status_text) setStatus(d.status_text);

    scrollToBottom(transcript);
    scrollToBottom(reviewLog);
    scrollToBottom(rigLog);

    setBusy(d.busy === true);
  }).catch(err => console.error('listen error:', err));

  try {
    const state = await invoke('get_initial_state');
    versionText.textContent    = state.version_text       ?? '';
    setStatus(state.status_text ?? '');
    inputEndpoint.value        = state.endpoint_text      ?? '';
    inputModel.value           = state.model_text         ?? '';
    inputApiKey.value          = state.api_key_text       ?? '';
    inputSysPrompt.value       = state.system_prompt_text ?? '';
    transcript.textContent     = state.transcript_text    ?? '';
    reviewLog.textContent      = state.review_log_text    ?? '';
    rigLog.textContent         = state.rig_log_text       ?? '';
    docsRigLog.textContent     = state.rig_log_text       ?? '';
    draftInput.value           = state.draft_message_text ?? '';
    docsStatusText.textContent = state.docs_status_text   ?? '';
    updateModelBadge();
  } catch (err) {
    setStatus(`Init error: ${err}`);
  }

  // Pre-load dashboard data
  refreshDashboard();
});
