// Tauri v2 IPC — accessed lazily inside handlers, never at module scope
function tauriApi() { return window.__TAURI__; }
function invoke(cmd, args) {
  const api = tauriApi();
  if (!api) return Promise.reject(new Error('Tauri IPC not ready'));
  return api.core.invoke(cmd, args);
}
function listen(event, handler) {
  const api = tauriApi();
  if (!api) return Promise.reject(new Error('Tauri IPC not ready'));
  return api.event.listen(event, handler);
}

// ── Autorun: idle countdown (10s) + stay-active ──────────────────────────────
const IDLE_TIMEOUT_MS = 10000;
let idleTimer = null;
let autorunMode = false;

function resetIdleTimer() {
  if (idleTimer) clearTimeout(idleTimer);
  if (!autorunMode) return;
  idleTimer = setTimeout(() => {
    document.title = document.title.replace(/ \[idle.*?\]/, '') + ' [idle: 0s]';
    let count = 0;
    const tick = setInterval(() => {
      count += 1;
      document.title = document.title.replace(/ \[idle: \d+s\]/, '') + ` [idle: ${count}s]`;
      if (count >= 10) {
        clearInterval(tick);
        document.title = document.title.replace(/ \[idle: \d+s\]/, ' [stay-active]');
      }
    }, 1000);
  }, IDLE_TIMEOUT_MS);
}

document.addEventListener('click', resetIdleTimer);
document.addEventListener('keydown', resetIdleTimer);

// ── Panel definitions (single source of truth) ────────────────────────────────
const PANELS = [
  { id: 'chat',    icon: 'AI', label: 'Chat' },
  { id: 'logs',    icon: 'LG', label: 'Logs' },
  { id: 'dash',    icon: 'DB', label: 'Dashboard' },
  { id: 'settings',icon: 'ST', label: 'Settings' },
  { id: 'docs',    icon: 'DK', label: 'Docs Playbook' },
];

// ── Generated template functions ──────────────────────────────────────────────
function panelTemplate(id) {
  const tpls = {
    chat: `
      <div class="panel-header">
        <span class="panel-title">Chat</span>
        <div id="model-badge" class="model-badge phi">
          <span id="model-badge-icon">&#9889;</span>
          <span id="model-badge-text">No model — go to Settings</span>
        </div>
      </div>
      <div class="model-bar">
        <span class="model-bar-label">Model:</span>
        <button id="pill-phi" class="model-pill" title="Switch to Internal Phi-4">&#9889; Internal Phi-4 Mini</button>
        <button id="pill-foundry" class="model-pill" title="Switch to Windows AI / Foundry Local">Windows AI / Foundry</button>
        <button id="pill-cloud" class="model-pill" title="Switch to Cloud / OpenAI">&#9729; Cloud / OpenAI</button>
        <span id="cloud-hint" class="cloud-hint hidden">— edit endpoint &amp; key in Settings</span>
      </div>
      <div id="transcript-wrap" class="transcript-wrap">
        <div class="log-label">Transcript</div>
        <div id="transcript" class="transcript-content"></div>
      </div>
      <div class="input-area">
        <textarea id="draft-input" rows="5" placeholder="Type a message..."></textarea>
        <div class="input-actions">
          <button id="send-btn">Send</button>
          <button id="rhai-btn">Rhai Rule Prompt</button>
        </div>
      </div>`,
    logs: `
      <div class="panel-title-row"><span class="panel-title">Logs</span></div>
      <div class="log-tabs">
        <button class="log-tab active" data-log="0">Transport</button>
        <button class="log-tab" data-log="1">Review</button>
      </div>
      <div id="log-panel-0" class="log-subpanel transport-bg">
        <div class="log-label">Rig / OpenAI Transport</div>
        <div id="rig-log" class="log-content"></div>
      </div>
      <div id="log-panel-1" class="log-subpanel review-bg hidden">
        <div class="log-label review-label">Review Diffsets</div>
        <div id="review-log" class="log-content"></div>
      </div>`,
    dash: `
      <span class="panel-title">Dashboard</span>
      <div id="evidence-summary" class="evidence-summary">
        <div class="ev-card ev-card-blocked">
          <div class="ev-card-value" id="blocked-value">-</div>
          <div class="ev-card-label">Blocked</div>
        </div>
        <div class="ev-card ev-card-ready">
          <div class="ev-card-value" id="ready-value">-</div>
          <div class="ev-card-label">Ready to Review</div>
        </div>
        <div class="ev-card ev-card-exported">
          <div class="ev-card-value" id="exported-value">-</div>
          <div class="ev-card-label">Exported</div>
        </div>
        <div class="ev-card ev-card-issues">
          <div class="ev-card-value" id="issues-value">-</div>
          <div class="ev-card-label">Validation Issues</div>
        </div>
      </div>
      <div class="ev-section">
        <div class="ev-section-title">Last Action</div>
        <div id="ev-last-action" class="ev-last-action">Loading...</div>
      </div>
      <div class="ev-section">
        <div class="ev-section-title">Next Actions</div>
        <ul id="ev-next-actions" class="ev-next-actions"></ul>
      </div>
      <div class="ev-section">
        <div class="ev-section-title">Provider Status</div>
        <div id="ev-provider-status" class="ev-provider-status">Loading...</div>
      </div>
      <div class="ev-refresh-row">
        <button id="btn-refresh-dashboard" class="ev-refresh-btn">Refresh Dashboard</button>
      </div>`,
    settings: `
      <span class="panel-title">Settings</span>
      <label class="field-label" for="input-endpoint">Endpoint URL</label>
      <input id="input-endpoint" type="text" class="field-input" />
      <label class="field-label" for="input-model">Model</label>
      <input id="input-model" type="text" class="field-input" />
      <label class="field-label" for="input-api-key">API Key</label>
      <input id="input-api-key" type="text" class="field-input" />
      <label class="field-label" for="input-system-prompt">System Prompt</label>
      <textarea id="input-system-prompt" class="field-input system-prompt-area" rows="6"></textarea>
      <div class="settings-actions">
        <button id="btn-use-phi">Use Internal Phi-4</button>
        <button id="btn-use-foundry">Use Windows AI</button>
        <button id="btn-use-cloud">Use Cloud Model</button>
        <button id="btn-save-settings">Save Settings</button>
      </div>`,
    docs: `
      <span class="panel-title">Docs Playbook</span>
      <p id="docs-status-text" class="docs-status"></p>
      <div class="docs-actions">
        <button id="btn-open-docs">Open Docs Playbook</button>
        <button id="btn-load-rhai-mutation">Load Rhai Mutation Prompt</button>
      </div>
      <div class="docs-preview-wrap">
        <div id="docs-rig-log" class="log-content"></div>
      </div>`,
  };
  return tpls[id] || '';
}

// ── Generate DOM from PANELS ──────────────────────────────────────────────────
function buildUI() {
  const navContainer = document.getElementById('nav-items');
  const panelContainer = document.getElementById('panel-container');

  PANELS.forEach((p, i) => {
    // sidebar button
    const btn = document.createElement('button');
    btn.className = 'nav-item';
    btn.dataset.panelIndex = i;
    btn.innerHTML = `<span class="mark">${p.icon}</span><span class="label">${p.label}</span>`;
    btn.addEventListener('click', () => showPanel(i));
    navContainer.appendChild(btn);

    // panel div
    const div = document.createElement('div');
    div.id = `panel-${p.id}`;
    div.className = `panel card${i === 0 ? '' : ' hidden'}`;
    if (p.id === 'settings') div.classList.add('settings-bg');
    div.innerHTML = panelTemplate(p.id);
    panelContainer.appendChild(div);
  });
}

// ── State ─────────────────────────────────────────────────────────────────────
let activePanel = 0;
let activeLogPanel = 0;
let busy = false;
autorunMode = true; // enable autorun countdown on startup

// ── DOM references (populated after buildUI) ──────────────────────────────────
let sidebar, collapseBtn, navItems, panels;
let versionText, statusBar;
let modelBadge, modelBadgeIcon, modelBadgeText;
let pillPhi, pillFoundry, pillCloud, cloudHint;
let transcript, draftInput, sendBtn, rhaiBtn;
let rigLog, reviewLog, docsRigLog, logTabs, logPanels;
let inputEndpoint, inputModel, inputApiKey, inputSysPrompt;
let btnUsePhi, btnUseFoundry, btnUseCloud, btnSave;
let docsStatusText, btnOpenDocs, btnLoadRhai;
let btnRefreshDash, blockedValue, readyValue, exportedValue, issuesValue;
let evLastAction, evNextActions, evProviderStat;

function cacheRefs() {
  sidebar        = document.getElementById('sidebar');
  collapseBtn    = document.getElementById('collapse-btn');
  navItems       = document.querySelectorAll('.nav-item[data-panel-index]');
  panels         = PANELS.map(p => document.getElementById(`panel-${p.id}`));

  versionText    = document.getElementById('version-text');
  statusBar      = document.getElementById('status-bar');

  modelBadge     = document.getElementById('model-badge');
  modelBadgeIcon = document.getElementById('model-badge-icon');
  modelBadgeText = document.getElementById('model-badge-text');

  pillPhi        = document.getElementById('pill-phi');
  pillFoundry    = document.getElementById('pill-foundry');
  pillCloud      = document.getElementById('pill-cloud');
  cloudHint      = document.getElementById('cloud-hint');

  transcript     = document.getElementById('transcript');
  draftInput     = document.getElementById('draft-input');
  sendBtn        = document.getElementById('send-btn');
  rhaiBtn        = document.getElementById('rhai-btn');

  rigLog         = document.getElementById('rig-log');
  reviewLog      = document.getElementById('review-log');
  docsRigLog     = document.getElementById('docs-rig-log');
  logTabs        = document.querySelectorAll('.log-tab');
  logPanels      = [0,1].map(i => document.getElementById(`log-panel-${i}`));

  inputEndpoint  = document.getElementById('input-endpoint');
  inputModel     = document.getElementById('input-model');
  inputApiKey    = document.getElementById('input-api-key');
  inputSysPrompt = document.getElementById('input-system-prompt');
  btnUsePhi      = document.getElementById('btn-use-phi');
  btnUseFoundry  = document.getElementById('btn-use-foundry');
  btnUseCloud    = document.getElementById('btn-use-cloud');
  btnSave        = document.getElementById('btn-save-settings');

  docsStatusText = document.getElementById('docs-status-text');
  btnOpenDocs    = document.getElementById('btn-open-docs');
  btnLoadRhai    = document.getElementById('btn-load-rhai-mutation');

  btnRefreshDash = document.getElementById('btn-refresh-dashboard');
  blockedValue   = document.getElementById('blocked-value');
  readyValue     = document.getElementById('ready-value');
  exportedValue  = document.getElementById('exported-value');
  issuesValue    = document.getElementById('issues-value');
  evLastAction   = document.getElementById('ev-last-action');
  evNextActions  = document.getElementById('ev-next-actions');
  evProviderStat = document.getElementById('ev-provider-status');
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function setStatus(text) {
  statusBar.textContent = text;
}

function setBusy(val) {
  busy = val;
  const toggles = [sendBtn, rhaiBtn, draftInput, pillPhi, pillFoundry, pillCloud,
    btnUsePhi, btnUseFoundry, btnUseCloud, btnSave, btnOpenDocs, btnLoadRhai,
    inputEndpoint, inputModel, inputApiKey, inputSysPrompt, btnRefreshDash];
  toggles.forEach(el => { if (el) el.disabled = val; });
  if (sendBtn) sendBtn.textContent = val ? 'Sending…' : 'Send';
  if (btnSave) btnSave.textContent = val ? 'Working…' : 'Save Settings';
}

function showPanel(index) {
  activePanel = index;
  panels.forEach((p, i) => p.classList.toggle('hidden', i !== index));
  navItems.forEach(item => {
    item.classList.toggle('active', parseInt(item.dataset.panelIndex, 10) === index);
  });
  if (index === dashIndex()) refreshDashboard();
}

function dashIndex() { return PANELS.findIndex(p => p.id === 'dash'); }

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
  if (el) el.scrollTop = el.scrollHeight;
}

// ── Sidebar collapse ──────────────────────────────────────────────────────────

collapseBtn.addEventListener('click', () => {
  sidebar.classList.toggle('collapsed');
  collapseBtn.querySelector('.mark').textContent =
    sidebar.classList.contains('collapsed') ? '>' : '<';
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
  } catch (err) { setStatus(`Error: ${err}`); }
});

// ── Model pills ───────────────────────────────────────────────────────────────

pillPhi.addEventListener('click',     async () => { if (busy) return; try { applyChatSettings(await invoke('use_internal_phi',     { systemPrompt: inputSysPrompt.value })); } catch (e) { setStatus(`Error: ${e}`); }});
pillCloud.addEventListener('click',   async () => { if (busy) return; try { applyChatSettings(await invoke('use_cloud_model',      { systemPrompt: inputSysPrompt.value })); } catch (e) { setStatus(`Error: ${e}`); }});
pillFoundry.addEventListener('click', async () => { if (busy) return; try { applyChatSettings(await invoke('use_foundry_local',    { systemPrompt: inputSysPrompt.value })); } catch (e) { setStatus(`Error: ${e}`); }});
btnUsePhi.addEventListener('click',   async () => { if (busy) return; try { applyChatSettings(await invoke('use_internal_phi',     { systemPrompt: inputSysPrompt.value })); } catch (e) { setStatus(`Error: ${e}`); }});
btnUseFoundry.addEventListener('click', async () => { if (busy) return; try { applyChatSettings(await invoke('use_foundry_local',   { systemPrompt: inputSysPrompt.value })); } catch (e) { setStatus(`Error: ${e}`); }});
btnUseCloud.addEventListener('click', async () => { if (busy) return; try { applyChatSettings(await invoke('use_cloud_model',      { systemPrompt: inputSysPrompt.value })); } catch (e) { setStatus(`Error: ${e}`); }});

// ── Chat: Send ────────────────────────────────────────────────────────────────

async function sendMessage() {
  if (busy) return;
  const draft = draftInput.value;
  if (!draft.trim()) { setStatus('Enter a message before sending.'); return; }
  setBusy(true);
  try {
    setStatus(await invoke('send_message', {
      draft, endpoint: inputEndpoint.value, model: inputModel.value,
      apiKey: inputApiKey.value, systemPrompt: inputSysPrompt.value,
    }));
  } catch (err) { setStatus(`Error: ${err}`); setBusy(false); }
}

sendBtn.addEventListener('click', sendMessage);
draftInput.addEventListener('keydown', e => {
  if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) sendMessage();
});

// ── Chat: Rhai Rule Prompt ────────────────────────────────────────────────────

rhaiBtn.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('load_rhai_rule_prompt', {
      currentModel: inputModel.value, currentSystemPrompt: inputSysPrompt.value,
    });
    inputSysPrompt.value  = payload.system_prompt;
    if (payload.suggested_model) inputModel.value = payload.suggested_model;
    draftInput.value      = payload.draft_message;
    reviewLog.textContent = payload.review_log_text;
    scrollToBottom(reviewLog);
    setStatus(payload.status);
    updateModelBadge();
  } catch (err) { setStatus(`Error: ${err}`); }
});

// ── Docs Playbook ─────────────────────────────────────────────────────────────

btnOpenDocs.addEventListener('click', async () => {
  if (busy) return;
  try { setStatus(await invoke('open_docs_playbook')); } catch (err) { setStatus(`Error: ${err}`); }
});

btnLoadRhai.addEventListener('click', async () => {
  if (busy) return;
  try {
    const payload = await invoke('load_rhai_rule_prompt', {
      currentModel: inputModel.value, currentSystemPrompt: inputSysPrompt.value,
    });
    inputSysPrompt.value  = payload.system_prompt;
    if (payload.suggested_model) inputModel.value = payload.suggested_model;
    draftInput.value      = payload.draft_message;
    reviewLog.textContent = payload.review_log_text;
    scrollToBottom(reviewLog);
    setStatus(payload.status);
    updateModelBadge();
    showPanel(PANELS.findIndex(p => p.id === 'chat'));
  } catch (err) { setStatus(`Error: ${err}`); }
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
  } catch (err) { evLastAction.textContent = `Error: ${err}`; }
}

btnRefreshDash.addEventListener('click', refreshDashboard);

// ── Initialise on load ────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', async () => {
  try {
    buildUI();
  } catch (e) {
    console.error('buildUI failed:', e);
    document.getElementById('panel-container').innerHTML = `<div class="panel card" style="padding:20px"><h2>UI Load Error</h2><pre>${e.message}\n${e.stack}</pre></div>`;
    return;
  }
  try {
    cacheRefs();
  } catch (e) {
    console.error('cacheRefs failed:', e);
    return;
  }
  showPanel(0);
  showLogPanel(0);

  listen('chat-update', event => {
    const d = event.payload;
    transcript.textContent = d.transcript_text  ?? '';
    reviewLog.textContent  = d.review_log_text  ?? '';
    rigLog.textContent     = d.rig_log_text     ?? '';
    docsRigLog.textContent = d.rig_log_text     ?? '';
    if (typeof d.draft_message_text === 'string') draftInput.value = d.draft_message_text;
    if (d.status_text) setStatus(d.status_text);
    scrollToBottom(transcript);
    scrollToBottom(reviewLog);
    scrollToBottom(rigLog);
    setBusy(d.busy === true);
  }).catch(err => console.error('listen error:', err));

  try {
    const state = await invoke('get_initial_state');
    versionText.textContent    = state.version_text ?? '';
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
  } catch (err) { setStatus(`Init error: ${err}`); }

  refreshDashboard();
});
