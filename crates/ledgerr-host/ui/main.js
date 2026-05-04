function tauriApi(){return window.__TAURI__}
function invoke(cmd,args){var api=window.__TAURI__;if(!api)return Promise.reject(new Error('no __TAURI__'));if(!api.core)return Promise.reject(new Error('no .core'));return api.core.invoke(cmd,args)}
function listen(e,h){var api=window.__TAURI__;if(!api)return Promise.reject(new Error('no __TAURI__'));return api.event.listen(e,h)}

var PANELS=[
  {id:'chat',icon:'AI',label:'Chat'},
  {id:'logs',icon:'LG',label:'Logs'},
  {id:'dash',icon:'DB',label:'Dashboard'},
  {id:'settings',icon:'ST',label:'Settings'},
  {id:'docs',icon:'DK',label:'Docs Playbook'},
];
var activePanel=0;
var DASH_PANEL_INDEX=PANELS.findIndex(function(p){return p.id==='dash'});

function showPanel(i){
  activePanel=i;
  PANELS.forEach(function(p,j){
    var el=document.getElementById('panel-'+p.id);
    if(el)el.classList.toggle('hidden',j!==i);
  });
  document.querySelectorAll('.nav-item[data-panel-index]').forEach(function(b,j){
    b.classList.toggle('active',j===i);
  });
  if(DASH_PANEL_INDEX!==-1&&i===DASH_PANEL_INDEX)refreshDashboard();
}

function panelTemplate(id){
  var t={}
  t.chat='<div class="panel-header"><span class="panel-title">Chat</span><div id="model-badge" class="model-badge phi"><span id="model-badge-icon">&#9889;</span><span id="model-badge-text">No model</span></div></div><div class="model-bar"><span class="model-bar-label">Model:</span><button id="pill-phi" class="model-pill">&#9889; Phi-4</button><button id="pill-foundry" class="model-pill">Windows AI</button><button id="pill-cloud" class="model-pill">&#9729; Cloud</button><span id="cloud-hint" class="cloud-hint hidden">edit in Settings</span></div><div id="transcript-wrap" class="transcript-wrap"><div class="log-label">Transcript</div><div id="transcript" class="transcript-content"></div></div><div class="input-area"><textarea id="draft-input" rows="5"></textarea><div class="input-actions"><button id="send-btn">Send</button><button id="rhai-btn">Rhai Rule</button></div></div>';
  t.logs='<div class="panel-title-row"><span class="panel-title">Logs</span></div><div class="log-tabs"><button class="log-tab active" data-log="0">Transport</button><button class="log-tab" data-log="1">Review</button></div><div id="log-panel-0" class="log-subpanel transport-bg"><div class="log-label">Transport</div><div id="rig-log" class="log-content"></div></div><div id="log-panel-1" class="log-subpanel review-bg hidden"><div class="log-label review-label">Diffsets</div><div id="review-log" class="log-content"></div></div></div>';
  t.dash='<span class="panel-title">Dashboard</span><div id="evidence-summary" class="evidence-summary"><div class="ev-card ev-card-blocked"><div class="ev-card-value" id="blocked-value">-</div><div class="ev-card-label">Blocked</div></div><div class="ev-card ev-card-ready"><div class="ev-card-value" id="ready-value">-</div><div class="ev-card-label">Ready</div></div><div class="ev-card ev-card-exported"><div class="ev-card-value" id="exported-value">-</div><div class="ev-card-label">Exported</div></div><div class="ev-card ev-card-issues"><div class="ev-card-value" id="issues-value">-</div><div class="ev-card-label">Issues</div></div></div><div class="ev-section"><div class="ev-section-title">Last Action</div><div id="ev-last-action" class="ev-last-action">Loading...</div></div><div class="ev-section"><div class="ev-section-title">Next Actions</div><ul id="ev-next-actions" class="ev-next-actions"></ul></div><div class="ev-section"><div class="ev-section-title">Providers</div><div id="ev-provider-status" class="ev-provider-status">Loading...</div></div><div class="ev-refresh-row"><button id="btn-refresh-dashboard">Refresh</button></div>';
  t.settings='<span class="panel-title">Settings</span><label class="field-label" for="input-endpoint">Endpoint</label><input id="input-endpoint" type="text" class="field-input"/><label class="field-label" for="input-model">Model</label><input id="input-model" type="text" class="field-input"/><label class="field-label" for="input-api-key">Key</label><input id="input-api-key" type="text" class="field-input"/><label class="field-label" for="input-system-prompt">System Prompt</label><textarea id="input-system-prompt" class="field-input system-prompt-area" rows="6"></textarea><div class="settings-actions"><button id="btn-use-phi">Use Phi-4</button><button id="btn-use-foundry">Use Win AI</button><button id="btn-use-cloud">Use Cloud</button><button id="btn-save-settings">Save</button></div>';
  t.docs='<span class="panel-title">Docs Playbook</span><p id="docs-status-text" class="docs-status"></p><div class="docs-actions"><button id="btn-open-docs">Open Docs</button><button id="btn-load-rhai-mutation">Load Rhai</button></div><div class="docs-preview-wrap"><div id="docs-rig-log" class="log-content"></div></div>';
  return t[id]||'';
}

function buildUI(){
  try{
    var nav=document.getElementById('nav-items');
    var pc=document.getElementById('panel-container');
    if(!nav||!pc)return;
    PANELS.forEach(function(p,i){
      var btn=document.createElement('button');btn.className='nav-item';btn.dataset.panelIndex=i;
      btn.innerHTML='<span class="mark">'+p.icon+'</span><span class="label">'+p.label+'</span>';
      (function(idx){btn.addEventListener('click',function(){showPanel(idx);});})(i);
      nav.appendChild(btn);
      var div=document.createElement('div');div.id='panel-'+p.id;
      div.className='panel card'+(i===0?'':' hidden');
      if(p.id==='settings')div.classList.add('settings-bg');
      div.innerHTML=panelTemplate(p.id);
      pc.appendChild(div);
    });
    showPanel(0);
  }catch(e){console.error('[ui] buildUI err:',e)}
}

function readinessLabel(r){
  if(!r)return'Unknown';
  if(r==='ready')return'Ready';
  if(r.setup_needed)return'Setup needed';
  if(r.unavailable)return'Unavailable';
  if(r.diagnostic)return'Diagnostic';
  return String(r);
}

function setTextSafe(el,text){
  if(el)el.textContent=text!=null?String(text):'';
}

function refreshDashboard(){
  var api=window.__TAURI__;
  if(!api)return;
  api.core.invoke('get_evidence_dashboard').then(function(p){
    var q=p.today_queue||{};
    setTextSafe(document.getElementById('blocked-value'),q.blocked??'-');
    setTextSafe(document.getElementById('ready-value'),q.ready_to_review??'-');
    setTextSafe(document.getElementById('exported-value'),q.exported??'-');
    setTextSafe(document.getElementById('issues-value'),q.with_validation_issues??'-');
    setTextSafe(document.getElementById('ev-last-action'),q.last_action_summary??'');
    var na=document.getElementById('ev-next-actions');
    if(na){
      na.innerHTML='';
      (q.next_actions||[]).forEach(function(a){
        var li=document.createElement('li');
        li.textContent=a;
        na.appendChild(li);
      });
    }
    var ps=document.getElementById('ev-provider-status');
    if(ps){
      ps.innerHTML='';
      (q.providers||[]).forEach(function(prov){
        var d=document.createElement('div');
        d.className='ev-provider-line';
        d.textContent=`${prov.display_name||prov.label}: ${readinessLabel(prov.readiness)}`;
        ps.appendChild(d);
      });
    }
  }).catch(function(err){
    var sb=document.getElementById('status-bar');
    if(sb)sb.textContent='Dashboard refresh failed: '+(err&&err.message||err||'unknown error');
  });
}

document.addEventListener('DOMContentLoaded',function(){
  buildUI();
  refreshDashboard();
  // Wire dashboard refresh button
  var dr=document.getElementById('btn-refresh-dashboard');
  if(dr)dr.addEventListener('click',refreshDashboard);
  // Wire settings
  var sf=document.getElementById('btn-save-settings');
  if(sf)sf.addEventListener('click',function(){
    var ep=document.getElementById('input-endpoint');var mo=document.getElementById('input-model');
    var ak=document.getElementById('input-api-key');var sp=document.getElementById('input-system-prompt');
    invoke('save_settings',{endpoint:ep?.value||'',model:mo?.value||'',apiKey:ak?.value||'',systemPrompt:sp?.value||''}).then(function(s){var sb=document.getElementById('status-bar');if(sb)sb.textContent=s}).catch(function(){});
  });
});
