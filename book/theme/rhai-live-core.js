(function (root, factory) {
    const api = factory();
    if (typeof module !== "undefined" && module.exports) {
        module.exports = api;
    }
    if (root) {
        root.RhaiLiveCore = api;
    }
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
    function sanitizeId(raw) {
        return raw.replace(/[^A-Za-z0-9_]/g, "_");
    }

    function escapeLabel(raw) {
        return raw.replace(/"/g, "&quot;");
    }

    function parseRhaiDiagram(source) {
        const graph = {
            order: [],
            nodes: new Map(),
            edges: [],
        };
        const pipelineEdges = [];
        const conditionals = [];
        const diagnostics = [];

        function addNode(id, label, kind) {
            if (!graph.nodes.has(id)) {
                graph.order.push(id);
                graph.nodes.set(id, { id, label, kind });
            }
        }

        function addEdge(from, to, label) {
            graph.edges.push({ from, to, label: label || null });
        }

        function parseCondition(expr, target) {
            const operators = [">=", "<=", "!=", ">", "<", "=="];
            for (const op of operators) {
                const index = expr.indexOf(op);
                if (index !== -1) {
                    const lhs = expr.slice(0, index).trim();
                    const rhs = expr.slice(index + op.length).trim();
                    if (lhs && rhs) {
                        return { lhs, op, rhs, target };
                    }
                }
            }
            return null;
        }

        function emitThresholdChain(lhs, op, thresholds) {
            const nodeIds = thresholds.map(([value]) =>
                sanitizeId(`${lhs}_${opWord(op)}_${String(value).replace(/\./g, "_")}`)
            );

            thresholds.forEach(([value, target], index) => {
                const nodeId = nodeIds[index];
                addNode(nodeId, `${lhs} ${op} ${value}`, "decision");

                const targetId = sanitizeId(target);
                addNode(targetId, target, "step");
                addEdge(nodeId, targetId, "true");

                if (index + 1 < thresholds.length) {
                    addEdge(nodeId, nodeIds[index + 1], "false");
                }
            });
        }

        function opWord(op) {
            switch (op) {
                case ">":
                    return "gt";
                case "<":
                    return "lt";
                case ">=":
                    return "gte";
                case "<=":
                    return "lte";
                case "==":
                    return "eq";
                case "!=":
                    return "ne";
                default:
                    return "op";
            }
        }

        source.split("\n").forEach((rawLine, index) => {
            const commentIndex = rawLine.indexOf("//");
            const line = (commentIndex === -1 ? rawLine : rawLine.slice(0, commentIndex)).trim();
            if (!line) {
                return;
            }

            if (line.startsWith("fn ")) {
                const rest = line.slice(3);
                const parts = rest.split("->");
                if (parts.length === 2) {
                    const name = parts[0].trim().replace(/\(\)\s*$/, "").trim();
                    const target = parts[1].trim();
                    if (name && target) {
                        pipelineEdges.push([name, target]);
                        return;
                    }
                }
                diagnostics.push({
                    line: index + 1,
                    kind: "error",
                    message: "Malformed fn edge; expected `fn source() -> target`.",
                    source: rawLine.trim(),
                });
                return;
            }

            if (line.startsWith("if ")) {
                const rest = line.slice(3);
                const parts = rest.split("->");
                if (parts.length === 2) {
                    const expr = parts[0].trim();
                    const target = parts[1].trim();
                    if (expr && target) {
                        const parsed = parseCondition(expr, target);
                        if (parsed) {
                            conditionals.push(parsed);
                        } else {
                            conditionals.push({
                                lhs: sanitizeId(expr),
                                op: "",
                                rhs: "",
                                target: sanitizeId(target),
                            });
                            diagnostics.push({
                                line: index + 1,
                                kind: "warning",
                                message:
                                    "Condition parsed as raw decision node; prefer operators like >, <, >=, <=, ==, !=",
                                source: rawLine.trim(),
                            });
                        }
                        return;
                    }
                }
                diagnostics.push({
                    line: index + 1,
                    kind: "error",
                    message: "Malformed if edge; expected `if expression -> target`.",
                    source: rawLine.trim(),
                });
                return;
            }

            diagnostics.push({
                line: index + 1,
                kind: "info",
                message:
                    "Line ignored by diagram DSL. Supported forms are `fn source() -> target` and `if expression -> target`.",
                source: rawLine.trim(),
            });
        });

        pipelineEdges.forEach(([name, target]) => {
            const nameId = sanitizeId(name);
            const targetId = sanitizeId(target);
            addNode(nameId, name, "step");
            addNode(targetId, target, "step");
            addEdge(nameId, targetId, null);
        });

        const gtGroups = new Map();
        const ltGroups = new Map();
        const plainConditions = [];

        conditionals.forEach((cond) => {
            if (cond.op === ">" && !Number.isNaN(Number(cond.rhs))) {
                const list = gtGroups.get(cond.lhs) || [];
                list.push([Number(cond.rhs), cond.target]);
                gtGroups.set(cond.lhs, list);
                return;
            }

            if (cond.op === "<" && !Number.isNaN(Number(cond.rhs))) {
                const list = ltGroups.get(cond.lhs) || [];
                list.push([Number(cond.rhs), cond.target]);
                ltGroups.set(cond.lhs, list);
                return;
            }

            plainConditions.push(cond);
        });

        gtGroups.forEach((thresholds, lhs) => {
            thresholds.sort((left, right) => right[0] - left[0]);
            emitThresholdChain(lhs, ">", thresholds);
        });

        ltGroups.forEach((thresholds, lhs) => {
            thresholds.sort((left, right) => left[0] - right[0]);
            emitThresholdChain(lhs, "<", thresholds);
        });

        plainConditions.forEach((cond) => {
            if (!cond.op) {
                const condId = cond.lhs;
                const targetId = sanitizeId(cond.target);
                addNode(condId, cond.lhs, "decision");
                addNode(targetId, cond.target, "step");
                addEdge(condId, targetId, null);
                return;
            }

            const exprLabel = `${cond.lhs} ${cond.op} ${cond.rhs}`;
            const condId = sanitizeId(exprLabel);
            const targetId = sanitizeId(cond.target);
            addNode(condId, exprLabel, "decision");
            addNode(targetId, cond.target, "step");
            addEdge(condId, targetId, "true");
        });

        return { graph, diagnostics };
    }

    function graphToMermaid(graph) {
        const lines = ["flowchart TD"];

        graph.order.forEach((id) => {
            const node = graph.nodes.get(id);
            if (!node) {
                return;
            }
            if (node.kind === "decision") {
                lines.push(`    ${node.id}{"${escapeLabel(node.label)}"}`);
            } else {
                lines.push(`    ${node.id}["${escapeLabel(node.label)}"]`);
            }
        });

        graph.edges.forEach((edge) => {
            if (edge.label) {
                lines.push(`    ${edge.from} -->|"${edge.label}"|${edge.to}`);
            } else {
                lines.push(`    ${edge.from} --> ${edge.to}`);
            }
        });

        return lines.join("\n");
    }

    return {
        sanitizeId,
        parseRhaiDiagram,
        graphToMermaid,
    };
});
