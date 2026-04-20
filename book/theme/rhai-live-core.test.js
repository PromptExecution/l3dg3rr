const test = require("node:test");
const assert = require("node:assert/strict");
const core = require("./rhai-live-core.js");

test("parses simple fn pipeline", function () {
    const result = core.parseRhaiDiagram("fn ingest() -> classify\nfn classify() -> commit\n");
    assert.equal(result.graph.nodes.size, 3);
    assert.equal(result.graph.edges.length, 2);
    assert.equal(result.diagnostics.length, 0);
});

test("emits diagnostics for unsupported and malformed lines", function () {
    const src = [
        "let x = 1",
        "fn bad_syntax() => nope",
        "if confidence > 0.8 -> commit",
    ].join("\n");
    const result = core.parseRhaiDiagram(src);
    assert.equal(result.graph.edges.length, 1);
    assert.ok(result.diagnostics.length >= 2);
    assert.ok(result.diagnostics.some((d) => d.kind === "info"));
    assert.ok(result.diagnostics.some((d) => d.kind === "error"));
});

test("builds threshold chain with false branch", function () {
    const src = "if confidence > 0.5 -> review\nif confidence > 0.8 -> commit\n";
    const result = core.parseRhaiDiagram(src);
    const mermaid = core.graphToMermaid(result.graph);
    assert.ok(mermaid.includes('|"false"|'));
    assert.ok(mermaid.includes("commit"));
    assert.ok(mermaid.includes("review"));
});
