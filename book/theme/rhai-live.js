(function () {
    const ORIGINAL_CLASS = "rhai-diagram-original";
    const MERMAID_CDN = "https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js";

    function escapeHtml(raw) {
        return raw
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    async function ensureMermaid() {
        if (window.mermaid) {
            return window.mermaid;
        }

        if (!window.__rhaiLiveMermaidLoadPromise) {
            window.__rhaiLiveMermaidLoadPromise = new Promise(function (resolve, reject) {
                const script = document.createElement("script");
                script.src = MERMAID_CDN;
                script.async = true;
                script.onload = function () {
                    if (window.mermaid) {
                        resolve(window.mermaid);
                    } else {
                        reject(new Error("Mermaid script loaded but window.mermaid is still undefined."));
                    }
                };
                script.onerror = function () {
                    reject(
                        new Error(
                            "Mermaid failed to load from CDN. Check network access or include Mermaid in mdBook assets."
                        )
                    );
                };
                document.head.appendChild(script);
            });
        }

        return window.__rhaiLiveMermaidLoadPromise;
    }

    async function renderMermaid(target, mermaidSource) {
        const mermaid = await ensureMermaid();
        if (!window.__rhaiLiveMermaidInitialized) {
            mermaid.initialize({ startOnLoad: false });
            window.__rhaiLiveMermaidInitialized = true;
        }

        const renderId = `rhai-live-${Math.random().toString(36).slice(2)}`;
        const rendered = await mermaid.render(renderId, mermaidSource);
        target.innerHTML = rendered.svg;
        if (typeof rendered.bindFunctions === "function") {
            rendered.bindFunctions(target);
        }
    }

    function diagnosticsHtml(diagnostics) {
        if (!diagnostics.length) {
            return "";
        }

        const lines = diagnostics.map(function (diag) {
            const cls =
                diag.kind === "error"
                    ? "rhai-diag-error"
                    : diag.kind === "warning"
                      ? "rhai-diag-warning"
                      : "rhai-diag-info";
            return `<li class="${cls}"><strong>L${diag.line}</strong> ${escapeHtml(diag.message)}<br><code>${escapeHtml(
                diag.source
            )}</code></li>`;
        });

        return `<div class="rhai-diag-panel"><div class="rhai-diag-title">Diagnostics</div><ul>${lines.join(
            ""
        )}</ul></div>`;
    }

    async function attachEditor(sourcePre) {
        const sourceCode = sourcePre && sourcePre.querySelector("code.language-rhai");
        const previewPre = sourcePre.nextElementSibling;
        if (!sourceCode || !previewPre || !previewPre.classList.contains("mermaid")) {
            return;
        }

        const core = window.RhaiLiveCore;
        if (!core) {
            return;
        }

        const shell = document.createElement("section");
        shell.className = "rhai-diagram-shell";

        const toolbar = document.createElement("div");
        toolbar.className = "rhai-diagram-toolbar";

        const status = document.createElement("div");
        status.className = "rhai-diagram-status";
        status.textContent = "Live Rhai diagram editor";

        const actions = document.createElement("div");
        actions.className = "rhai-diagram-actions";

        const regenerate = document.createElement("button");
        regenerate.type = "button";
        regenerate.className = "rhai-diagram-button";
        regenerate.textContent = "Regenerate";

        const reset = document.createElement("button");
        reset.type = "button";
        reset.className = "rhai-diagram-button";
        reset.textContent = "Reset";

        actions.appendChild(regenerate);
        actions.appendChild(reset);
        toolbar.appendChild(status);
        toolbar.appendChild(actions);

        const body = document.createElement("div");
        body.className = "rhai-diagram-body";

        const editor = document.createElement("textarea");
        editor.className = "rhai-diagram-editor";
        editor.spellcheck = false;
        editor.value = sourceCode.textContent || "";

        const preview = document.createElement("div");
        preview.className = "rhai-diagram-preview";

        const note = document.createElement("div");
        note.className = "rhai-diagram-note";
        note.textContent = "Edit the supported Rhai diagram DSL (`fn ... -> ...`, `if ... -> ...`) and regenerate.";

        body.appendChild(editor);
        body.appendChild(preview);
        shell.appendChild(toolbar);
        shell.appendChild(body);
        shell.appendChild(note);

        sourcePre.classList.add(ORIGINAL_CLASS);
        previewPre.classList.add(ORIGINAL_CLASS);
        previewPre.insertAdjacentElement("afterend", shell);

        const originalSource = editor.value;

        async function update() {
            try {
                const result = core.parseRhaiDiagram(editor.value);
                const graph = result.graph;
                const diagnostics = result.diagnostics;

                if (!graph.order.length && !graph.edges.length) {
                    preview.innerHTML =
                        '<p class="rhai-diagram-error">No diagramable Rhai DSL lines found in this block.</p>' +
                        diagnosticsHtml(diagnostics);
                    status.textContent = "No parseable diagram nodes";
                    return;
                }

                const mermaidSource = core.graphToMermaid(graph);
                await renderMermaid(preview, mermaidSource);
                preview.insertAdjacentHTML("beforeend", diagnosticsHtml(diagnostics));
                status.textContent = `Rendered ${graph.nodes.size} nodes and ${graph.edges.length} edges`;
            } catch (error) {
                preview.innerHTML =
                    `<p class="rhai-diagram-error">${escapeHtml(error.message)}</p>` +
                    '<p class="rhai-diag-hint">Try refreshing, or verify Mermaid network availability.</p>';
                status.textContent = "Render failed";
            }
        }

        regenerate.addEventListener("click", update);
        reset.addEventListener("click", async function () {
            editor.value = originalSource;
            await update();
        });
        editor.addEventListener("keydown", async function (event) {
            if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
                event.preventDefault();
                await update();
            }
        });

        await update();
    }

    async function main() {
        const sourceBlocks = Array.from(document.querySelectorAll("pre"));
        for (const sourcePre of sourceBlocks) {
            const code = sourcePre.querySelector("code.language-rhai");
            if (!code) {
                continue;
            }

            const previewPre = sourcePre.nextElementSibling;
            if (!previewPre || !previewPre.classList.contains("mermaid")) {
                continue;
            }

            await attachEditor(sourcePre);
        }
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", function () {
            main().catch((error) => console.error("rhai-live:", error));
        });
    } else {
        main().catch((error) => console.error("rhai-live:", error));
    }
})();
