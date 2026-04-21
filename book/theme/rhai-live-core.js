(function (root, factory) {
    const api = factory();
    if (typeof module !== "undefined" && module.exports) {
        module.exports = api;
    }
    if (root) {
        root.RhaiLiveCore = api;
    }
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
    const ISO_SETTINGS = {
        levelGap: 192,
        laneGap: 136,
        decisionLift: 34,
        reviewLift: 18,
        commitLift: 12,
        scale: 1,
        margin: 64,
        cardWidth: 122,
        cardHeight: 68,
        cardDepth: 20,
        animationMs: 460,
    };

    const ICON_LIBRARY = {
        step: {
            name: "step",
            viewBox: "0 0 24 24",
            path: "M5 5H19V19H5Z M8 9H16 M8 12H16 M8 15H13",
            polygon: [
                [-0.75, -0.55],
                [0.75, -0.55],
                [0.75, 0.55],
                [-0.75, 0.55],
            ],
        },
        ingest: {
            name: "ingest",
            viewBox: "0 0 24 24",
            path: "M6 7H18V10H20V18H4V10H6Z M12 4V13 M8.5 9.5L12 13L15.5 9.5",
            polygon: [
                [-0.7, -0.55],
                [0.7, -0.55],
                [0.7, -0.1],
                [0.3, -0.1],
                [0.3, 0.7],
                [-0.3, 0.7],
                [-0.3, -0.1],
                [-0.7, -0.1],
            ],
        },
        validate: {
            name: "validate",
            viewBox: "0 0 24 24",
            path: "M12 4L18 6.5V11.5C18 15 15.5 18.25 12 20C8.5 18.25 6 15 6 11.5V6.5Z M9.5 12.5L11.2 14.2L14.8 10.6",
            polygon: [
                [0, -0.9],
                [0.78, -0.45],
                [0.65, 0.55],
                [0, 0.95],
                [-0.65, 0.55],
                [-0.78, -0.45],
            ],
        },
        classify: {
            name: "classify",
            viewBox: "0 0 24 24",
            path: "M4 8L12 4L20 8L12 12Z M4 12L12 16L20 12 M4 16L12 20L20 16",
            polygon: [
                [0, -0.9],
                [0.82, -0.28],
                [0.48, 0.82],
                [-0.48, 0.82],
                [-0.82, -0.28],
            ],
        },
        review: {
            name: "review",
            viewBox: "0 0 24 24",
            path: "M3.5 12C5.6 8.1 8.5 6.2 12 6.2C15.5 6.2 18.4 8.1 20.5 12C18.4 15.9 15.5 17.8 12 17.8C8.5 17.8 5.6 15.9 3.5 12 Z M12 9.5A2.5 2.5 0 1 0 12 14.5A2.5 2.5 0 1 0 12 9.5",
            polygon: [
                [-0.95, 0],
                [-0.55, -0.55],
                [0, -0.78],
                [0.55, -0.55],
                [0.95, 0],
                [0.55, 0.55],
                [0, 0.78],
                [-0.55, 0.55],
            ],
        },
        reconcile: {
            name: "reconcile",
            viewBox: "0 0 24 24",
            path: "M12 5V18 M7 8H17 M6 8L4.5 11H7.5Z M16.5 11L18 8L19.5 11Z M8 8V14 M16 8V14 M5 14H11 M13 14H19",
            polygon: [
                [-0.18, -0.9],
                [0.18, -0.9],
                [0.18, -0.1],
                [0.85, 0.7],
                [0.42, 0.92],
                [0, 0.45],
                [-0.42, 0.92],
                [-0.85, 0.7],
                [-0.18, -0.1],
            ],
        },
        commit: {
            name: "commit",
            viewBox: "0 0 24 24",
            path: "M12 4A8 8 0 1 0 12 20A8 8 0 1 0 12 4 M8.8 12.2L11.1 14.5L15.7 9.9",
            polygon: regularPolygon(20, 0.85),
        },
        decision: {
            name: "decision",
            viewBox: "0 0 24 24",
            path: "M12 4L19.5 12L12 20L4.5 12Z M12 8V12 M12 15H12.01",
            polygon: [
                [0, -0.95],
                [0.95, 0],
                [0, 0.95],
                [-0.95, 0],
            ],
        },
    };

    function regularPolygon(sides, radius) {
        const points = [];
        for (let index = 0; index < sides; index += 1) {
            const angle = -Math.PI / 2 + (index / sides) * Math.PI * 2;
            points.push([Number((Math.cos(angle) * radius).toFixed(4)), Number((Math.sin(angle) * radius).toFixed(4))]);
        }
        return points;
    }

    function sanitizeId(raw) {
        return raw.replace(/[^A-Za-z0-9_]/g, "_");
    }

    function escapeHtml(raw) {
        return String(raw)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    function escapeLabel(raw) {
        return escapeHtml(raw).replace(/"/g, "&quot;");
    }

    function inferSemanticType(label, kind) {
        const normalized = String(label || "").toLowerCase();
        if (kind === "decision" || kind === "match") {
            return "decision";
        }
        if (/\b(ingest|load|parse|extract|source|input)\b/.test(normalized)) {
            return "ingest";
        }
        if (/\b(validate|verify|check|guard|audit|rule)\b/.test(normalized)) {
            return "validate";
        }
        if (/\b(classify|label|tag|map|route)\b/.test(normalized)) {
            return "classify";
        }
        if (/\b(review|approve|manual|operator|human)\b/.test(normalized)) {
            return "review";
        }
        if (/\b(reconcile|match|balance|ledger)\b/.test(normalized)) {
            return "reconcile";
        }
        if (/\b(commit|publish|export|write|persist|done|finish)\b/.test(normalized)) {
            return "commit";
        }
        return "step";
    }

    function createNodeRecord(id, label, kind, armIndex, isDefault) {
        return {
            id,
            identity_key: id,
            label,
            kind,
            semanticType: inferSemanticType(label, kind),
            arm_index: armIndex !== undefined ? armIndex : null,
            is_default: !!isDefault,
        };
    }

    function isDefaultArm(armLabel) {
        const trimmed = String(armLabel || "").trim();
        return trimmed === "_" || trimmed === "else" || trimmed === "otherwise" || trimmed === "default";
    }

    function parseRhaiDiagram(source) {
        const graph = {
            order: [],
            nodes: new Map(),
            edges: [],
        };
        const pipelineEdges = [];
        const conditionals = [];
        const matchArms = [];
        const diagnostics = [];

        function addNode(id, label, kind, armIndex, isDefault) {
            if (!graph.nodes.has(id)) {
                graph.order.push(id);
                graph.nodes.set(id, createNodeRecord(id, label, kind, armIndex, isDefault));
            }
        }

        function addEdge(from, to, label, armIndex, isDefault) {
            graph.edges.push({ from, to, label: label || null, arm_index: armIndex !== undefined ? armIndex : null, is_default: !!isDefault });
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

        String(source)
            .split("\n")
            .forEach(function (rawLine, index) {
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
                                        "Condition parsed as a raw decision node; prefer operators like >, <, >=, <=, ==, !=",
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

                if (line.startsWith("match ")) {
                    const rest = line.slice(6);
                    const matchParts = rest.split("=>");
                    if (matchParts.length === 2) {
                        const expr = matchParts[0].trim();
                        const armTarget = matchParts[1].split("->");
                        if (armTarget.length === 2) {
                            const arm = armTarget[0].trim();
                            const target = armTarget[1].trim();
                            if (expr && arm && target) {
                                matchArms.push({ expr, arm, target });
                                return;
                            }
                        }
                    }
                    diagnostics.push({
                        line: index + 1,
                        kind: "error",
                        message: "Malformed match arm; expected `match expr => Arm -> target`.",
                        source: rawLine.trim(),
                    });
                    return;
                }

                diagnostics.push({
                    line: index + 1,
                    kind: "info",
                    message:
                        "Line ignored by diagram DSL. Supported forms are `fn source() -> target`, `if expression -> target`, and `match expr => Arm -> target`.",
                    source: rawLine.trim(),
                });
            });

        pipelineEdges.forEach(function ([name, target]) {
            const nameId = sanitizeId(name);
            const targetId = sanitizeId(target);
            addNode(nameId, name, "step");
            addNode(targetId, target, "step");
            addEdge(nameId, targetId, null);
        });

        const gtGroups = new Map();
        const ltGroups = new Map();
        const plainConditions = [];

        conditionals.forEach(function (cond) {
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

        gtGroups.forEach(function (thresholds, lhs) {
            thresholds.sort(function (left, right) {
                return right[0] - left[0];
            });
            emitThresholdChain(lhs, ">", thresholds);
        });

        ltGroups.forEach(function (thresholds, lhs) {
            thresholds.sort(function (left, right) {
                return left[0] - right[0];
            });
            emitThresholdChain(lhs, "<", thresholds);
        });

        plainConditions.forEach(function (cond) {
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

        const matchGroups = new Map();
        const matchOrder = [];

        matchArms.forEach(function (entry) {
            if (!matchGroups.has(entry.expr)) {
                matchGroups.set(entry.expr, []);
                matchOrder.push(entry.expr);
            }
            matchGroups.get(entry.expr).push(entry);
        });

        matchOrder.forEach(function (expr) {
            const nodeId = `match_${sanitizeId(expr)}`;
            addNode(nodeId, `match ${expr}`, "match");
            const arms = matchGroups.get(expr);
            arms.forEach(function (entry, armIndex) {
                const targetId = sanitizeId(entry.target);
                const isDefault = isDefaultArm(entry.arm);
                addNode(targetId, entry.target, "step", armIndex, isDefault);
                addEdge(nodeId, targetId, entry.arm, armIndex, isDefault);
            });
        });

        return { graph, diagnostics };
    }

    function graphToMermaid(graph) {
        const lines = ["flowchart TD"];

        graph.order.forEach(function (id) {
            const node = graph.nodes.get(id);
            if (!node) {
                return;
            }
            if (node.kind === "decision" || node.kind === "match") {
                lines.push(`    ${node.id}{"${escapeLabel(node.label)}"}`);
            } else {
                lines.push(`    ${node.id}["${escapeLabel(node.label)}"]`);
            }
        });

        graph.edges.forEach(function (edge) {
            let label = edge.label;
            if (edge.is_default && label) {
                label = label + " (default)";
            } else if (edge.is_default && !label) {
                label = "_";
            }
            if (label) {
                lines.push(`    ${edge.from} -->|"${escapeLabel(label)}"|${edge.to}`);
            } else {
                lines.push(`    ${edge.from} --> ${edge.to}`);
            }
        });

        return lines.join("\n");
    }

    function graphTopology(graph) {
        const incoming = new Map();
        const outgoing = new Map();
        const indexById = new Map();
        graph.order.forEach(function (id, index) {
            incoming.set(id, []);
            outgoing.set(id, []);
            indexById.set(id, index);
        });

        graph.edges.forEach(function (edge) {
            if (!incoming.has(edge.to)) {
                incoming.set(edge.to, []);
            }
            if (!outgoing.has(edge.from)) {
                outgoing.set(edge.from, []);
            }
            incoming.get(edge.to).push(edge);
            outgoing.get(edge.from).push(edge);
        });

        return { incoming, outgoing, indexById };
    }

    function computeLevels(graph, topology) {
        const indegree = new Map();
        graph.order.forEach(function (id) {
            indegree.set(id, topology.incoming.get(id).length);
        });

        const queue = graph.order.filter(function (id) {
            return indegree.get(id) === 0;
        });
        const levels = new Map();
        const visited = new Set();

        while (queue.length) {
            queue.sort(function (left, right) {
                return topology.indexById.get(left) - topology.indexById.get(right);
            });
            const id = queue.shift();
            if (visited.has(id)) {
                continue;
            }
            visited.add(id);
            const preds = topology.incoming.get(id);
            const level = preds.length
                ? Math.max.apply(
                      null,
                      preds.map(function (edge) {
                          return (levels.get(edge.from) || 0) + 1;
                      })
                  )
                : 0;
            levels.set(id, level);

            topology.outgoing.get(id).forEach(function (edge) {
                indegree.set(edge.to, (indegree.get(edge.to) || 0) - 1);
                if (indegree.get(edge.to) <= 0) {
                    queue.push(edge.to);
                }
            });
        }

        graph.order.forEach(function (id) {
            if (!levels.has(id)) {
                const preds = topology.incoming.get(id);
                levels.set(
                    id,
                    preds.length
                        ? Math.max.apply(
                              null,
                              preds.map(function (edge) {
                                  return (levels.get(edge.from) || 0) + 1;
                              })
                          )
                        : 0
                );
            }
        });

        return levels;
    }

    function groupByLevel(graph, levels, topology) {
        const grouped = new Map();
        graph.order.forEach(function (id) {
            const level = levels.get(id) || 0;
            const list = grouped.get(level) || [];
            list.push(id);
            grouped.set(level, list);
        });

        grouped.forEach(function (list) {
            list.sort(function (left, right) {
                return topology.indexById.get(left) - topology.indexById.get(right);
            });
        });

        return grouped;
    }

    function refineLevelOrder(grouped, topology) {
        const levelKeys = Array.from(grouped.keys()).sort(function (left, right) {
            return left - right;
        });

        function barycenter(nodeId, edges, lookup) {
            const related = edges.get(nodeId);
            if (!related || !related.length) {
                return null;
            }
            const values = related
                .map(function (edge) {
                    return lookup.get(edge.from || edge.to);
                })
                .filter(function (value) {
                    return typeof value === "number";
                });
            if (!values.length) {
                return null;
            }
            return values.reduce(function (sum, value) {
                return sum + value;
            }, 0) / values.length;
        }

        for (let pass = 0; pass < 2; pass += 1) {
            const forwardOrder = new Map();
            levelKeys.forEach(function (level) {
                const previous = grouped.get(level - 1) || [];
                previous.forEach(function (id, index) {
                    forwardOrder.set(id, index);
                });
                const current = grouped.get(level) || [];
                current.sort(function (left, right) {
                    const leftScore = barycenter(left, topology.incoming, forwardOrder);
                    const rightScore = barycenter(right, topology.incoming, forwardOrder);
                    if (leftScore === null && rightScore === null) {
                        return topology.indexById.get(left) - topology.indexById.get(right);
                    }
                    if (leftScore === null) {
                        return 1;
                    }
                    if (rightScore === null) {
                        return -1;
                    }
                    return leftScore - rightScore;
                });
            });

            const backwardOrder = new Map();
            [...levelKeys].reverse().forEach(function (level) {
                const next = grouped.get(level + 1) || [];
                next.forEach(function (id, index) {
                    backwardOrder.set(id, index);
                });
                const current = grouped.get(level) || [];
                current.sort(function (left, right) {
                    const leftScore = barycenter(
                        left,
                        new Map(
                            Array.from(topology.outgoing.entries()).map(function ([key, edges]) {
                                return [
                                    key,
                                    edges.map(function (edge) {
                                        return { from: edge.to };
                                    }),
                                ];
                            })
                        ),
                        backwardOrder
                    );
                    const rightScore = barycenter(
                        right,
                        new Map(
                            Array.from(topology.outgoing.entries()).map(function ([key, edges]) {
                                return [
                                    key,
                                    edges.map(function (edge) {
                                        return { from: edge.to };
                                    }),
                                ];
                            })
                        ),
                        backwardOrder
                    );
                    if (leftScore === null && rightScore === null) {
                        return topology.indexById.get(left) - topology.indexById.get(right);
                    }
                    if (leftScore === null) {
                        return 1;
                    }
                    if (rightScore === null) {
                        return -1;
                    }
                    return leftScore - rightScore;
                });
            });
        }

        return levelKeys;
    }

    function clamp(value, min, max) {
        return Math.min(max, Math.max(min, value));
    }

    function median(values) {
        if (!values.length) {
            return 0;
        }
        const sorted = values.slice().sort(function (left, right) {
            return left - right;
        });
        const mid = Math.floor(sorted.length / 2);
        if (sorted.length % 2 === 0) {
            return (sorted[mid - 1] + sorted[mid]) / 2;
        }
        return sorted[mid];
    }

    function solveConstraintLayout(graph, options) {
        const settings = Object.assign({}, ISO_SETTINGS, options || {});
        const topology = graphTopology(graph);
        const levels = computeLevels(graph, topology);
        const grouped = groupByLevel(graph, levels, topology);
        const levelKeys = refineLevelOrder(grouped, topology);
        const positions = new Map();

        levelKeys.forEach(function (level) {
            const ids = grouped.get(level) || [];
            const center = (ids.length - 1) / 2;
            ids.forEach(function (id, index) {
                const node = graph.nodes.get(id);
                const lift =
                    node.kind === "decision" || node.kind === "match"
                        ? settings.decisionLift
                        : node.semanticType === "review"
                          ? settings.reviewLift
                          : node.semanticType === "commit"
                            ? settings.commitLift
                            : 0;
                positions.set(id, {
                    x: level * settings.levelGap,
                    y: lift,
                    z: (index - center) * settings.laneGap,
                });
            });
        });

        for (let pass = 0; pass < 6; pass += 1) {
            levelKeys.forEach(function (level) {
                const ids = (grouped.get(level) || []).slice().sort(function (left, right) {
                    return positions.get(left).z - positions.get(right).z;
                });

                ids.forEach(function (id, index) {
                    const pos = positions.get(id);
                    const incoming = topology.incoming.get(id);
                    if (incoming.length) {
                        const targetZ = median(
                            incoming.map(function (edge) {
                                return positions.get(edge.from).z;
                            })
                        );
                        pos.z += (targetZ - pos.z) * 0.16;
                    }

                    const outgoing = topology.outgoing.get(id);
                    if (outgoing.length > 1) {
                        const spread = (index - (ids.length - 1) / 2) * 8;
                        pos.z += spread;
                    }
                });

                for (let index = 1; index < ids.length; index += 1) {
                    const prev = positions.get(ids[index - 1]);
                    const curr = positions.get(ids[index]);
                    const minGap = settings.laneGap * 0.78;
                    const gap = curr.z - prev.z;
                    if (gap < minGap) {
                        const correction = (minGap - gap) / 2;
                        prev.z -= correction;
                        curr.z += correction;
                    }
                }
            });
        }

        return {
            positions,
            levels,
            grouped,
            settings,
        };
    }

    function isoProject(point, scale, origin) {
        return {
            x: origin.x + (point.x - point.z) * scale * 0.866,
            y: origin.y + (point.x + point.z) * scale * 0.5 - point.y * scale,
        };
    }

    function colorForType(type) {
        switch (type) {
            case "ingest":
                return "#1d4ed8";
            case "validate":
                return "#0f766e";
            case "classify":
                return "#7c3aed";
            case "review":
                return "#b45309";
            case "reconcile":
                return "#2563eb";
            case "commit":
                return "#15803d";
            case "decision":
                return "#b91c1c";
            default:
                return "#475569";
        }
    }

    function tint(color, amount) {
        const normalized = color.replace("#", "");
        const rgb = [0, 2, 4].map(function (offset) {
            return parseInt(normalized.slice(offset, offset + 2), 16);
        });
        const adjusted = rgb.map(function (value) {
            return clamp(Math.round(value + (255 - value) * amount), 0, 255);
        });
        return (
            "#" +
            adjusted
                .map(function (value) {
                    return value.toString(16).padStart(2, "0");
                })
                .join("")
        );
    }

    function darken(color, amount) {
        const normalized = color.replace("#", "");
        const rgb = [0, 2, 4].map(function (offset) {
            return parseInt(normalized.slice(offset, offset + 2), 16);
        });
        const adjusted = rgb.map(function (value) {
            return clamp(Math.round(value * (1 - amount)), 0, 255);
        });
        return (
            "#" +
            adjusted
                .map(function (value) {
                    return value.toString(16).padStart(2, "0");
                })
                .join("")
        );
    }

    function flattenPath(path) {
        return String(path).replace(/\s+/g, " ").trim();
    }

    function polygonToPath(points, scale) {
        const parts = points.map(function ([x, y], index) {
            const px = Number((x * scale).toFixed(2));
            const py = Number((y * scale).toFixed(2));
            return `${index === 0 ? "M" : "L"} ${px} ${py}`;
        });
        return `${parts.join(" ")} Z`;
    }

    function encodeBase64(raw) {
        if (typeof Buffer !== "undefined") {
            return Buffer.from(raw, "utf8").toString("base64");
        }
        if (typeof btoa !== "undefined") {
            return btoa(unescape(encodeURIComponent(raw)));
        }
        throw new Error("No base64 encoder available.");
    }

    function encodeBase64Bytes(bytes) {
        if (typeof Buffer !== "undefined") {
            return Buffer.from(bytes).toString("base64");
        }
        if (typeof btoa !== "undefined") {
            let binary = "";
            bytes.forEach(function (byte) {
                binary += String.fromCharCode(byte);
            });
            return btoa(binary);
        }
        throw new Error("No base64 encoder available.");
    }

    function concatUint8Arrays(chunks) {
        const total = chunks.reduce(function (sum, chunk) {
            return sum + chunk.byteLength;
        }, 0);
        const result = new Uint8Array(total);
        let offset = 0;
        chunks.forEach(function (chunk) {
            result.set(new Uint8Array(chunk.buffer || chunk), offset);
            offset += chunk.byteLength;
        });
        return result;
    }

    function float32Bytes(values) {
        return new Uint8Array(new Float32Array(values).buffer);
    }

    function uint16Bytes(values) {
        return new Uint8Array(new Uint16Array(values).buffer);
    }

    function extrudeConvexPolygon(points, depth) {
        const positions = [];
        const indices = [];
        const half = depth / 2;

        points.forEach(function ([x, y]) {
            positions.push(x, y, half);
        });
        points.forEach(function ([x, y]) {
            positions.push(x, y, -half);
        });

        for (let index = 1; index < points.length - 1; index += 1) {
            indices.push(0, index, index + 1);
            indices.push(points.length, points.length + index + 1, points.length + index);
        }

        for (let index = 0; index < points.length; index += 1) {
            const next = (index + 1) % points.length;
            const frontA = index;
            const frontB = next;
            const backA = points.length + index;
            const backB = points.length + next;
            indices.push(frontA, frontB, backB);
            indices.push(frontA, backB, backA);
        }

        return { positions, indices };
    }

    function buildGltfDataUri(icon, color) {
        const mesh = extrudeConvexPolygon(icon.polygon || ICON_LIBRARY.step.polygon, 0.28);
        const positionBytes = float32Bytes(mesh.positions);
        const indexBytes = uint16Bytes(mesh.indices);
        const bufferBytes = concatUint8Arrays([positionBytes, indexBytes]);
        const bufferBase64 = encodeBase64Bytes(bufferBytes);
        const min = [-1, -1, -0.14];
        const max = [1, 1, 0.14];
        const json = {
            asset: { version: "2.0", generator: "rhai-live-core" },
            scenes: [{ nodes: [0] }],
            nodes: [{ mesh: 0, name: `${icon.name}-autogen` }],
            meshes: [
                {
                    name: `${icon.name}-mesh`,
                    primitives: [
                        {
                            attributes: { POSITION: 0 },
                            indices: 1,
                            extras: {
                                color,
                                icon: icon.name,
                                svgPath: flattenPath(icon.path),
                                autogenerated: true,
                            },
                        },
                    ],
                },
            ],
            accessors: [
                {
                    bufferView: 0,
                    componentType: 5126,
                    count: mesh.positions.length / 3,
                    type: "VEC3",
                    min,
                    max,
                },
                {
                    bufferView: 1,
                    componentType: 5123,
                    count: mesh.indices.length,
                    type: "SCALAR",
                },
            ],
            bufferViews: [
                {
                    buffer: 0,
                    byteOffset: 0,
                    byteLength: positionBytes.byteLength,
                    target: 34962,
                },
                {
                    buffer: 0,
                    byteOffset: positionBytes.byteLength,
                    byteLength: indexBytes.byteLength,
                    target: 34963,
                },
            ],
            buffers: [
                {
                    uri: `data:application/octet-stream;base64,${bufferBase64}`,
                    byteLength: bufferBytes.byteLength,
                },
            ],
        };

        return `data:model/gltf+json;base64,${encodeBase64(JSON.stringify(json))}`;
    }

    function iconForNode(node) {
        return ICON_LIBRARY[node.semanticType] || ICON_LIBRARY.step;
    }

    function buildVisualizationModel(graph, options) {
        const layout = solveConstraintLayout(graph, options);
        const settings = layout.settings;
        const origin = { x: settings.margin + settings.cardWidth / 2, y: settings.margin + 140 };
        const projected = [];

        graph.order.forEach(function (id) {
            const point = layout.positions.get(id);
            const screen = isoProject(point, settings.scale, origin);
            projected.push({ id, point, screen });
        });

        const minX = Math.min.apply(
            null,
            projected.map(function (entry) {
                return entry.screen.x - settings.cardWidth;
            })
        );
        const maxX = Math.max.apply(
            null,
            projected.map(function (entry) {
                return entry.screen.x + settings.cardWidth;
            })
        );
        const minY = Math.min.apply(
            null,
            projected.map(function (entry) {
                return entry.screen.y - settings.cardHeight;
            })
        );
        const maxY = Math.max.apply(
            null,
            projected.map(function (entry) {
                return entry.screen.y + settings.cardHeight + settings.cardDepth;
            })
        );
        const offset = {
            x: settings.margin - minX,
            y: settings.margin - minY,
        };

        const nodes = graph.order.map(function (id) {
            const node = graph.nodes.get(id);
            const point = layout.positions.get(id);
            const projectedPoint = isoProject(point, settings.scale, {
                x: origin.x + offset.x,
                y: origin.y + offset.y,
            });
            const icon = iconForNode(node);
            const color = colorForType(node.semanticType);
            return {
                id: node.id,
                label: node.label,
                kind: node.kind,
                semanticType: node.semanticType,
                x: point.x,
                y: point.y,
                z: point.z,
                level: layout.levels.get(id) || 0,
                screen: projectedPoint,
                color,
                icon,
                modelUri: buildGltfDataUri(icon, color),
            };
        });

        const nodeById = new Map(nodes.map(function (node) {
            return [node.id, node];
        }));

        const edges = graph.edges.map(function (edge, index) {
            const from = nodeById.get(edge.from);
            const to = nodeById.get(edge.to);
            const start = {
                x: from.screen.x + settings.cardWidth * 0.18,
                y: from.screen.y,
            };
            const end = {
                x: to.screen.x - settings.cardWidth * 0.18,
                y: to.screen.y,
            };
            const midX = (start.x + end.x) / 2;
            const bend = clamp(Math.abs(end.x - start.x) * 0.22, 32, 72);
            const path = [
                `M ${start.x.toFixed(1)} ${start.y.toFixed(1)}`,
                `C ${(midX - bend).toFixed(1)} ${(start.y + 12).toFixed(1)}, ${(midX + bend).toFixed(1)} ${(end.y - 12).toFixed(1)}, ${end.x.toFixed(1)} ${end.y.toFixed(1)}`,
            ].join(" ");
            return {
                id: `edge-${index}`,
                from: edge.from,
                to: edge.to,
                label: edge.label,
                path,
                labelPoint: {
                    x: midX,
                    y: (start.y + end.y) / 2 - 14,
                },
            };
        });

        return {
            settings,
            width: Math.ceil(maxX - minX + settings.margin * 2),
            height: Math.ceil(maxY - minY + settings.margin * 2),
            nodes,
            edges,
        };
    }

    function renderNodeIcon(icon, size) {
        const scale = size / 2;
        return `<path d="${flattenPath(icon.path)}" transform="translate(12 12) scale(${(scale / 12).toFixed(4)}) translate(-12 -12)" />`;
    }

    function faceOffsets(depth) {
        return {
            dx: depth * 0.86,
            dy: depth * -0.48,
        };
    }

    function renderStepNode(node, previousNode, settings) {
        const width = settings.cardWidth;
        const height = settings.cardHeight;
        const depth = settings.cardDepth;
        const { dx, dy } = faceOffsets(depth);
        const x = -width / 2;
        const y = -height / 2;
        const fill = node.color;
        const topFill = tint(fill, 0.18);
        const sideFill = darken(fill, 0.16);
        const frontFill = tint(fill, 0.08);
        const stroke = darken(fill, 0.34);
        const iconFill = "#f8fafc";
        const label = escapeHtml(node.label);
        const modelSafe = escapeHtml(node.modelUri);

        let animation = "";
        if (previousNode) {
            animation = `<animateTransform attributeName="transform" type="translate" from="${previousNode.screen.x.toFixed(1)} ${previousNode.screen.y.toFixed(1)}" to="${node.screen.x.toFixed(1)} ${node.screen.y.toFixed(1)}" dur="${settings.animationMs}ms" fill="freeze" calcMode="spline" keySplines=".2 .8 .2 1" />`;
        }

        return `<g class="rhai-iso-node rhai-iso-node-${node.semanticType}" data-node-id="${node.id}" data-model-uri="${modelSafe}" transform="translate(${node.screen.x.toFixed(1)} ${node.screen.y.toFixed(1)})">
            ${animation}
            <path class="rhai-iso-face-top" d="M ${x} ${y} L ${x + dx} ${y + dy} L ${x + width + dx} ${y + dy} L ${x + width} ${y} Z" fill="${topFill}" stroke="${stroke}" stroke-width="1.2" />
            <path class="rhai-iso-face-side" d="M ${x + width} ${y} L ${x + width + dx} ${y + dy} L ${x + width + dx} ${y + height + dy} L ${x + width} ${y + height} Z" fill="${sideFill}" stroke="${stroke}" stroke-width="1.2" />
            <rect class="rhai-iso-face-front" x="${x}" y="${y}" width="${width}" height="${height}" rx="16" fill="${frontFill}" stroke="${stroke}" stroke-width="1.4" />
            <circle cx="${x + 22}" cy="${y + 22}" r="13" fill="${fill}" />
            <g class="rhai-iso-icon" fill="${iconFill}" stroke="${iconFill}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <g transform="translate(${x + 10} ${y + 10})">${renderNodeIcon(node.icon, 24)}</g>
            </g>
            <text class="rhai-iso-label" x="${x + 42}" y="${y + 28}">${label}</text>
            <text class="rhai-iso-subtitle" x="${x + 42}" y="${y + 47}">${node.semanticType}</text>
        </g>`;
    }

    function renderDecisionNode(node, previousNode, settings) {
        const width = settings.cardWidth * 0.88;
        const height = settings.cardHeight * 0.9;
        const depth = settings.cardDepth;
        const { dx, dy } = faceOffsets(depth);
        const fill = node.color;
        const topFill = tint(fill, 0.18);
        const sideFill = darken(fill, 0.18);
        const frontFill = tint(fill, 0.1);
        const stroke = darken(fill, 0.34);
        const diamond = [
            [0, -height / 2],
            [width / 2, 0],
            [0, height / 2],
            [-width / 2, 0],
        ];
        const top = diamond.map(function ([x, y]) {
            return `${x + dx},${y + dy}`;
        });

        let animation = "";
        if (previousNode) {
            animation = `<animateTransform attributeName="transform" type="translate" from="${previousNode.screen.x.toFixed(1)} ${previousNode.screen.y.toFixed(1)}" to="${node.screen.x.toFixed(1)} ${node.screen.y.toFixed(1)}" dur="${settings.animationMs}ms" fill="freeze" calcMode="spline" keySplines=".2 .8 .2 1" />`;
        }

        return `<g class="rhai-iso-node rhai-iso-node-decision" data-node-id="${node.id}" data-model-uri="${escapeHtml(node.modelUri)}" transform="translate(${node.screen.x.toFixed(1)} ${node.screen.y.toFixed(1)})">
            ${animation}
            <polygon points="${diamond
                .map(function ([x, y]) {
                    return `${x},${y}`;
                })
                .join(" ")}" fill="${frontFill}" stroke="${stroke}" stroke-width="1.4" />
            <polygon points="${top.join(" ")}" fill="${topFill}" stroke="${stroke}" stroke-width="1.2" />
            <polygon points="${[
                `${diamond[1][0]},${diamond[1][1]}`,
                `${diamond[1][0] + dx},${diamond[1][1] + dy}`,
                `${diamond[2][0] + dx},${diamond[2][1] + dy}`,
                `${diamond[2][0]},${diamond[2][1]}`,
            ].join(" ")}" fill="${sideFill}" stroke="${stroke}" stroke-width="1.2" />
            <circle cx="0" cy="-2" r="14" fill="${fill}" />
            <g class="rhai-iso-icon" fill="#f8fafc" stroke="#f8fafc" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <g transform="translate(-12 -14)">${renderNodeIcon(node.icon, 24)}</g>
            </g>
            <text class="rhai-iso-label rhai-iso-label-center" x="0" y="${height / 2 + 18}">${escapeHtml(node.label)}</text>
        </g>`;
    }

    function sceneToIsometricSvg(scene, previousScene) {
        const previousById = new Map(
            (previousScene && previousScene.nodes ? previousScene.nodes : []).map(function (node) {
                return [node.id, node];
            })
        );

        const defs = `<defs>
            <pattern id="rhai-iso-grid" width="48" height="24" patternUnits="userSpaceOnUse" patternTransform="skewX(-30)">
                <path d="M 0 0 L 0 24 M 0 0 L 48 0" stroke="rgba(15, 23, 42, 0.08)" stroke-width="1" fill="none" />
            </pattern>
            <marker id="rhai-iso-arrow" markerWidth="9" markerHeight="9" refX="7" refY="4.5" orient="auto">
                <path d="M0,0 L9,4.5 L0,9 z" fill="#475569" />
            </marker>
        </defs>`;

        const edges = scene.edges
            .map(function (edge) {
                const labelWidth = edge.label ? Math.max(42, edge.label.length * 7.4) : 0;
                const label = edge.label
                    ? `<g class="rhai-iso-edge-label"><rect x="${(edge.labelPoint.x - labelWidth / 2).toFixed(1)}" y="${(edge.labelPoint.y - 11).toFixed(1)}" width="${labelWidth.toFixed(1)}" height="18" rx="9" /><text x="${edge.labelPoint.x.toFixed(1)}" y="${(edge.labelPoint.y + 1).toFixed(1)}">${escapeHtml(edge.label)}</text></g>`
                    : "";
                return `<g class="rhai-iso-edge"><path d="${edge.path}" marker-end="url(#rhai-iso-arrow)" />${label}</g>`;
            })
            .join("");

        const nodes = scene.nodes
            .map(function (node) {
                const previousNode = previousById.get(node.id);
                if (node.kind === "decision" || node.kind === "match") {
                    return renderDecisionNode(node, previousNode, scene.settings);
                }
                return renderStepNode(node, previousNode, scene.settings);
            })
            .join("");

        return `<svg class="rhai-isometric-scene" viewBox="0 0 ${scene.width} ${scene.height}" role="img" aria-label="Isometric Rhai workflow scene" xmlns="http://www.w3.org/2000/svg">
            ${defs}
            <rect class="rhai-iso-bg" width="${scene.width}" height="${scene.height}" rx="24" fill="url(#rhai-iso-grid)" />
            <g class="rhai-iso-shadow-plane" transform="translate(24 ${scene.height - 96})">
                <path d="M0 24 L${scene.width - 96} 24 L${scene.width - 48} 0 L48 0 Z" />
            </g>
            <g class="rhai-iso-layer-edges">${edges}</g>
            <g class="rhai-iso-layer-nodes">${nodes}</g>
        </svg>`;
    }

    function buildRenderFailure(error, viewMode) {
        const message = error && error.message ? error.message : "Unknown render failure.";
        if (viewMode === "mermaid-2d") {
            return {
                title: "Mermaid render failed",
                hint: "Switch to isometric-3d to keep inspecting the workflow while Mermaid is unavailable.",
                detail: message,
            };
        }
        return {
            title: "Isometric render failed",
            hint: "The graph parsed, but the isometric scene could not be generated. Mermaid remains available as a fallback.",
            detail: message,
        };
    }

    return {
        sanitizeId,
        escapeHtml,
        parseRhaiDiagram,
        graphToMermaid,
        buildVisualizationModel,
        sceneToIsometricSvg,
        buildRenderFailure,
        isoProject,
    };
});
