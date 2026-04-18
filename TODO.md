# TODO

## Phase 1: Notifier Core
- Add host-side `Notifier` trait and `NotificationStatus` model.
- Add `NotificationSettings` and backend selection policy.
- Implement `PowerShellBurntToastNotifier`.
- Add a tiny `notify-test` binary/command.
- Add tests for backend selection and command/result mapping.
- Independently validate a real Win11 toast from the command path.
- Check in with user after the first real toast and first green test pass.

## Phase 2: Persistent Settings
- Store non-secret operator settings in per-user JSON config.
- Persist immediately on sticky preference changes.
- Keep transient runtime health in memory only.
- Add atomic write/load fallback behavior.
- Add restart simulation tests for durability.
- Check in with user after restart persistence is proven.

## Phase 3: Tray State Model
- Define tray menu contract: `Toast Enabled`, `Test Toast`, `Status`, `Show Window`, `Quit`.
- Add typed tray commands and deterministic state rendering.
- Add tests for menu state and toggle behavior.

## Phase 4: Tray Runtime Integration
- Integrate `tray-icon`.
- Keep tray and Slint under one host event loop.
- Route tray/menu events into shared app state.
- Validate real tray icon lifecycle on Win11.

## Phase 5: Minimal Slint Window
- Add small settings/status window.
- Reflect notifier state and persisted settings.
- Keep window close distinct from app quit.

## Phase 6: Startup Wiring
- Load settings.
- Initialize notifier and probe health.
- Create tray.
- Create/show or hide Slint window from settings.
- Bind shared state and run event loop.

## Phase 7: Operator Event Log
- Record notification toggles, test requests/results, window restore, quit.
- Keep payloads stable and audit-friendly.

## Phase 8: Native Windows Notifier
- Add `WindowsNativeNotifier`.
- Preserve the same trait and status model.
- Keep PowerShell backend as dev/WSL fallback.

## Execution Loop
- Branch first.
- Work one phase at a time.
- Add or update tests in the same change.
- Independently validate tests before claiming done.
- Loop until tests pass.
- Check in with user at each gate.
- Memoize next steps in `AGENTS.md` and this file.
- Repeat until user is happy.

## Active Delegation
- Notifier track: own Phase 1 only.
- Settings track: own Phase 2 only.
- Tray track: own Phases 3 and 4 only.
- UI/integration track: own Phases 5 through 7 only.

## Sub-Agent Briefs

### Notifier
- Scope: Phase 1 only.
- Deliver: module/file layout, minimal trait/types, `notify-test` command shape, first tests.
- Constraints: no UI, no tray, no MCP coupling, no file edits outside notifier slice.

### Settings
- Scope: Phase 2 only.
- Deliver: config path choice, JSON schema, atomic write policy, restart test plan.
- Constraints: secrets stay out of settings; persist operator intent only.

### Tray
- Scope: Phases 3 and 4 only.
- Deliver: `tray-icon` integration plan, event-loop ownership model, menu state machine, first tray tests.
- Constraints: keep tray state separate from notifier backend details.

### UI/Integration
- Scope: Phases 5 through 7 only.
- Deliver: minimal Slint window plan, startup order, state sync model, audit event list.
- Constraints: keep close-to-tray distinct from quit; use shared app state.

## Phase 1 Note
- Proposed layout: `crates/ledgerr-host/src/notify/{mod.rs,types.rs,powershell.rs}` and `crates/ledgerr-host/src/bin/notify-test.rs`.
- Minimal types: `Notifier`, `NotificationStatus`, `NotificationSettings`, `NotificationEvent`, `NotificationTestResult`, `NotifierBackend`.
- Command shape: `cargo run -p ledgerr-host --bin notify-test -- --backend auto --title l3dg3rr --body "toast test"`.
- First tests: backend selection, disabled-path status, PowerShell command construction, result/status mapping, explicit `test()` event formatting.
