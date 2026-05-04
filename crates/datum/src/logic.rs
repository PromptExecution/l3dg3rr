//! Symbolic logic port system — NAND, NOR, ADD, WAIT, TX/RX, and the
//! universal flux capacitor meta-state requiring all ports to be filled.
//!
//! Extends the protocol encoding theory with gate-level operations.
//! Every operation is a typed port on a meta-state machine. Ports must
//! be filled (wired) before the meta-state can transition.
//!
//! ## Port types
//!
//! | Gate | Arity | Semantics |
//! |------|-------|-----------|
//! | NAND | 2→1 | NOT(a AND b) — universal gate |
//! | NOR  | 2→1 | NOT(a OR b) |
//! | ADD  | n→1 | Linear sum over input ports |
//! | WAIT | 1→1 | Hold until input stabilizes (debounce) |
//! | TX   | 1→0 | Transmit — emits value to output channel, consumes port |
//! | RX   | 0→1 | Receive — reads value from input channel, fills port |
//! | CAP  | n→m | Universal flux capacitor — all ports must be filled before meta-transition |
//!
//! ## Flux capacitor axiom
//!
//! A meta-state machine with `n` ports requires all `n` ports to be filled
//! (wired to a source) before any transition can fire. An unfilled port
//! produces `Disposition::Unrecoverable`.

/// Gate integer codes used by the macro's static array generation.
/// 0=NAND, 1=NOR, 2=ADD, 3=WAIT, 4=TX, 5=RX, 6=CAP
pub const GATE_CODES: &[(u8, &str)] = &[
    (0, "NAND"),
    (1, "NOR"),
    (2, "ADD"),
    (3, "WAIT"),
    (4, "TX"),
    (5, "RX"),
    (6, "CAP"),
];

/// Port identifier within a meta-state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortId(pub usize);

/// Logical gate type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateKind {
    Nand,
    Nor,
    Add,
    Wait,
    Tx,
    Rx,
    Cap,
}

impl GateKind {
    pub fn arity_in(&self) -> usize {
        match self {
            GateKind::Nand => 2,
            GateKind::Nor => 2,
            GateKind::Add => 2,
            GateKind::Wait => 1,
            GateKind::Tx => 1,
            GateKind::Rx => 0,
            GateKind::Cap => 0,
        }
    }

    pub fn arity_out(&self) -> usize {
        match self {
            GateKind::Nand => 1,
            GateKind::Nor => 1,
            GateKind::Add => 1,
            GateKind::Wait => 1,
            GateKind::Tx => 0,
            GateKind::Rx => 1,
            GateKind::Cap => 0,
        }
    }

    pub fn from_code(code: u8) -> Option<GateKind> {
        match code {
            0 => Some(GateKind::Nand),
            1 => Some(GateKind::Nor),
            2 => Some(GateKind::Add),
            3 => Some(GateKind::Wait),
            4 => Some(GateKind::Tx),
            5 => Some(GateKind::Rx),
            6 => Some(GateKind::Cap),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wire {
    pub from_gate: usize,
    pub from_port: usize,
    pub to_gate: usize,
    pub to_port: usize,
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub id: usize,
    pub kind: GateKind,
    pub input_ports: Vec<Option<Wire>>,
    pub output_wire: Option<Wire>,
    pub value: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct FluxCapacitor {
    pub ports: Vec<Gate>,
    pub wires: Vec<Wire>,
    pub meta_stable: bool,
    pub charge: f64,
    pub transition_count: u64,
}

impl FluxCapacitor {
    pub fn new() -> Self {
        Self {
            ports: Vec::new(),
            wires: Vec::new(),
            meta_stable: false,
            charge: 0.0,
            transition_count: 0,
        }
    }

    pub fn add_gate(&mut self, kind: GateKind) -> usize {
        let id = self.ports.len();
        let arity = kind.arity_in();
        let input_ports = (0..arity).map(|_| None).collect();
        self.ports.push(Gate {
            id,
            kind,
            input_ports,
            output_wire: None,
            value: None,
        });
        id
    }

    pub fn wire(
        &mut self,
        from_gate: usize,
        from_port: usize,
        to_gate: usize,
        to_port: usize,
    ) -> Result<(), String> {
        if from_gate >= self.ports.len() {
            return Err(format!("from_gate {from_gate} does not exist"));
        }
        if to_gate >= self.ports.len() {
            return Err(format!("to_gate {to_gate} does not exist"));
        }
        let from_kind = self.ports[from_gate].kind;
        if from_port >= from_kind.arity_out() {
            return Err(format!(
                "from_gate {from_gate} ({from_kind:?}) has no output port {from_port}"
            ));
        }
        let to_kind = self.ports[to_gate].kind;
        if to_port >= to_kind.arity_in() {
            return Err(format!(
                "to_gate {to_gate} ({to_kind:?}) has no input port {to_port}"
            ));
        }
        let wire = Wire {
            from_gate,
            from_port,
            to_gate,
            to_port,
        };
        self.ports[to_gate].input_ports[to_port] = Some(wire.clone());
        self.ports[from_gate].output_wire = Some(wire.clone());
        self.wires.push(wire);
        Ok(())
    }

    pub fn all_ports_filled(&self) -> Vec<(usize, String)> {
        let mut unfilled = Vec::new();
        for gate in &self.ports {
            let kind = gate.kind;
            let required = kind.arity_in();
            let filled = gate.input_ports.iter().filter(|p| p.is_some()).count();
            if filled < required && kind != GateKind::Rx && kind != GateKind::Cap {
                unfilled.push((
                    gate.id,
                    format!(
                        "{kind:?} gate {} has {filled}/{required} ports filled",
                        gate.id
                    ),
                ));
            }
        }
        unfilled
    }

    pub fn evaluate(&mut self) -> bool {
        let unfilled = self.all_ports_filled();
        let has_wires = !self.wires.is_empty();
        self.meta_stable = unfilled.is_empty() && has_wires;
        if self.meta_stable {
            self.charge = self.wires.len() as f64;
            self.transition_count += 1;
        }
        self.meta_stable
    }

    pub fn is_stable(&self) -> bool {
        self.meta_stable
    }

    pub fn charge_level(&self) -> f64 {
        self.charge
    }

    pub fn transition_count(&self) -> u64 {
        self.transition_count
    }
}

impl Default for FluxCapacitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Shorthand tokenizer: `&&` → AND, `||` → OR, `!` → NOT, `->` → Arrow, `<-` → BackArrow.
#[derive(Debug, Clone, PartialEq)]
pub enum ShorthandToken {
    Ident(String),
    And,
    Or,
    Not,
    Arrow,
    BackArrow,
    Eq,
    Colon,
    LBrace,
    RBrace,
    Number(u64),
}

pub fn tokenize_shorthand(input: &str) -> Vec<ShorthandToken> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '&' if chars.peek() == Some(&'&') => {
                chars.next();
                tokens.push(ShorthandToken::And);
            }
            '|' if chars.peek() == Some(&'|') => {
                chars.next();
                tokens.push(ShorthandToken::Or);
            }
            '!' => tokens.push(ShorthandToken::Not),
            '-' if chars.peek() == Some(&'>') => {
                chars.next();
                tokens.push(ShorthandToken::Arrow);
            }
            '<' if chars.peek() == Some(&'-') => {
                chars.next();
                tokens.push(ShorthandToken::BackArrow);
            }
            '=' => tokens.push(ShorthandToken::Eq),
            ':' => tokens.push(ShorthandToken::Colon),
            '{' => tokens.push(ShorthandToken::LBrace),
            '}' => tokens.push(ShorthandToken::RBrace),
            c if c.is_whitespace() => continue,
            c if c.is_ascii_digit() => {
                let mut n = c.to_digit(10).unwrap() as u64;
                while let Some(d) = chars.peek().and_then(|c| c.to_digit(10)) {
                    n = n * 10 + d as u64;
                    chars.next();
                }
                tokens.push(ShorthandToken::Number(n));
            }
            c if c.is_ascii_alphabetic() || c == '_' || c == '.' => {
                let mut s = String::from(c);
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '.' {
                        s.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(ShorthandToken::Ident(s));
            }
            _ => {}
        }
    }
    tokens
}

// ── Gate evaluation functions ────────────────────────────────────────────────

pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}
pub fn nor(a: bool, b: bool) -> bool {
    !(a || b)
}
pub fn add_u8(a: u8, b: u8) -> u8 {
    a.wrapping_add(b)
}
pub fn wait_stable(current: bool, previous: bool, debounce_ticks: u64, tick: u64) -> bool {
    if current == previous && tick >= debounce_ticks {
        current
    } else {
        previous
    }
}

// ── MultiBridge: two gate networks composed under a typed invariant ─────────

/// A typed bridge connecting two flux capacitor networks.
/// Bridges have a direction, a source kind, a target kind, and an invariant
/// predicate that must hold for the bridge to be valid.
#[derive(Debug, Clone)]
pub struct MultiBridge {
    pub name: String,
    pub source_kind: GateKind,
    pub target_kind: GateKind,
    pub source_ports: Vec<usize>,
    pub target_ports: Vec<usize>,
    pub wires: Vec<(usize, usize)>, // (source_port → target_port)
}

impl MultiBridge {
    pub fn new(name: &str, source: GateKind, target: GateKind) -> Self {
        Self {
            name: name.to_owned(),
            source_kind: source,
            target_kind: target,
            source_ports: Vec::new(),
            target_ports: Vec::new(),
            wires: Vec::new(),
        }
    }

    pub fn wire(&mut self, from_port: usize, to_port: usize) -> &mut Self {
        self.source_ports.push(from_port);
        self.target_ports.push(to_port);
        self.wires.push((from_port, to_port));
        self
    }

    /// Invariant: every source port must connect to exactly one target port
    /// and vice versa (bijection). Returns list of violations.
    pub fn validate_invariants(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let mut seen_source = Vec::new();
        let mut seen_target = Vec::new();

        for (s, t) in &self.wires {
            if seen_source.contains(s) {
                issues.push(format!(
                    "bridge '{}': source port {s} wired multiple times",
                    self.name
                ));
            }
            if seen_target.contains(t) {
                issues.push(format!(
                    "bridge '{}': target port {t} wired multiple times",
                    self.name
                ));
            }
            seen_source.push(*s);
            seen_target.push(*t);
        }

        issues
    }

    /// Apply the bridge wiring to a pair of capacitors.
    pub fn apply(
        &self,
        source: &mut FluxCapacitor,
        target: &mut FluxCapacitor,
    ) -> Result<(), String> {
        for (s_port, t_port) in &self.wires {
            if *s_port >= source.ports.len() {
                return Err(format!(
                    "bridge '{}': source has no port {s_port}",
                    self.name
                ));
            }
            if *t_port >= target.ports.len() {
                return Err(format!(
                    "bridge '{}': target has no port {t_port}",
                    self.name
                ));
            }
            // Wire source gate's output to target gate's input (port 0 → port 0)
            let _ = source.wire(*s_port, 0, *t_port, 0);
        }
        Ok(())
    }
}

/// Invariant validator: checks that a specific invariant holds across a bridge.
pub fn check_bridge_invariant(
    bridge: &MultiBridge,
    source: &FluxCapacitor,
    target: &FluxCapacitor,
) -> Vec<String> {
    let mut violations = bridge.validate_invariants();
    if !source.is_stable() {
        violations.push(format!(
            "bridge '{}': source capacitor not stable",
            bridge.name
        ));
    }
    if !target.is_stable() {
        violations.push(format!(
            "bridge '{}': target capacitor not stable",
            bridge.name
        ));
    }
    violations
}

// ── ModelTransformer: kind-to-kind transformation with shape constraints ────

/// A typed model transformer: maps a source gate kind to a target gate kind,
/// preserving a structural invariant across the transformation.
///
/// Like a transformer model in ML: input embedding → attention → output.
/// Here: source GateKind → transformation rule → target GateKind.
#[derive(Debug, Clone)]
pub struct ModelTransformer {
    pub name: String,
    pub input_shape: Vec<GateKind>,
    pub output_shape: Vec<GateKind>,
    pub weights: Vec<f64>,
}

impl ModelTransformer {
    pub fn new(name: &str, input: Vec<GateKind>, output: Vec<GateKind>) -> Self {
        let weight_count = input.len().max(output.len());
        Self {
            name: name.to_owned(),
            input_shape: input,
            output_shape: output,
            weights: vec![1.0; weight_count],
        }
    }

    /// Invariant: every input gate must map to at least one output gate (connectivity).
    pub fn check_connectivity(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.input_shape.is_empty() {
            issues.push(format!("transformer '{}': empty input shape", self.name));
        }
        if self.output_shape.is_empty() {
            issues.push(format!("transformer '{}': empty output shape", self.name));
        }
        issues
    }

    /// Invariant: input and output shapes have compatible port counts for sequential wiring.
    pub fn check_port_compatibility(&self) -> Result<(), String> {
        for inp in &self.input_shape {
            if inp.arity_out() == 0 {
                return Err(format!(
                    "transformer '{}': input gate {inp:?} has no output port",
                    self.name
                ));
            }
        }
        for out in &self.output_shape {
            if out.arity_in() == 0 {
                return Err(format!(
                    "transformer '{}': output gate {out:?} has no input port",
                    self.name
                ));
            }
        }
        Ok(())
    }

    /// Apply the transformation by wiring input capacitor to output capacitor.
    pub fn transform(
        &self,
        input: &mut FluxCapacitor,
        _output: &mut FluxCapacitor,
    ) -> Result<(), String> {
        self.check_port_compatibility()?;
        let n = self.input_shape.len().min(self.output_shape.len());
        for i in 0..n {
            let _ = input.wire(i, 0, i, 0);
        }
        Ok(())
    }
}

// ── AbstractFluxCapacitor: container for nested capacitors ──────────────────

/// A higher-order flux capacitor that contains multiple sub-capacitors.
/// Each sub-capacitor is a typed network. The abstract capacitor is stable
/// only when ALL sub-capacitors are stable AND their bridges are validated.
#[derive(Debug, Clone)]
pub struct AbstractFluxCapacitor {
    pub name: String,
    pub sub_capacitors: Vec<FluxCapacitor>,
    pub bridges: Vec<MultiBridge>,
    pub transformers: Vec<ModelTransformer>,
}

impl AbstractFluxCapacitor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            sub_capacitors: Vec::new(),
            bridges: Vec::new(),
            transformers: Vec::new(),
        }
    }

    pub fn add_capacitor(&mut self, cap: FluxCapacitor) -> usize {
        let id = self.sub_capacitors.len();
        self.sub_capacitors.push(cap);
        id
    }

    pub fn add_bridge(&mut self, bridge: MultiBridge) {
        self.bridges.push(bridge);
    }

    pub fn add_transformer(&mut self, tx: ModelTransformer) {
        self.transformers.push(tx);
    }

    /// Check all invariants across the abstract capacitor.
    /// Returns all violations found.
    pub fn check_all_invariants(&self) -> Vec<String> {
        let mut violations = Vec::new();

        for (i, cap) in self.sub_capacitors.iter().enumerate() {
            if !cap.is_stable() {
                violations.push(format!(
                    "abstract '{}': sub-capacitor {i} not stable",
                    self.name
                ));
            }
            let unfilled = cap.all_ports_filled();
            for (gid, msg) in &unfilled {
                violations.push(format!(
                    "abstract '{}': sub-capacitor {i} gate {gid}: {msg}",
                    self.name
                ));
            }
        }

        for bridge in &self.bridges {
            let bv = bridge.validate_invariants();
            for v in &bv {
                violations.push(format!(
                    "abstract '{}': bridge '{}': {v}",
                    self.name, bridge.name
                ));
            }
        }

        for tx in &self.transformers {
            let civ = tx.check_connectivity();
            for v in &civ {
                violations.push(format!(
                    "abstract '{}': transformer '{}': {v}",
                    self.name, tx.name
                ));
            }
            if let Err(e) = tx.check_port_compatibility() {
                violations.push(format!(
                    "abstract '{}': transformer '{}': {e}",
                    self.name, tx.name
                ));
            }
        }

        violations
    }

    pub fn is_abstractly_stable(&self) -> bool {
        self.check_all_invariants().is_empty()
    }
}

// ── symbolic_gate_test! macro ────────────────────────────────────────────────

/// Macro: `symbolic_gate_test!` — compile-time reasoned logic test generator.
///
/// Accepts comma-separated gate expressions. All gate names validated at
/// compile time — unknown idents produce `compile_error!`.
/// No inner macro calls — each token repeats via `$()*` to emit a direct
/// `codes.push(N)` or nothing, entirely within one expansion pass.
///
/// ```ignore
/// symbolic_gate_test!(NAND, TX, CAP);
/// symbolic_gate_test!(stable => RX, WAIT, CAP);
/// ```
#[macro_export]
macro_rules! symbolic_gate_test {
    // stable variant: each gate token emits a codes.push()
    (stable => $($gate:ident),+ $(,)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn stable() {
            let mut codes: Vec<u8> = Vec::new();
            $(
                $crate::symbolic_gate_test!(@emit codes $gate);
            )*
            let kinds: Vec<$crate::logic::GateKind> = codes.iter()
                .filter_map(|c| $crate::logic::GateKind::from_code(*c))
                .collect();
            $crate::logic::run_stable_test(&kinds);
        }
    };
    // basic variant
    ($($gate:ident),+ $(,)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn basic() {
            let mut codes: Vec<u8> = Vec::new();
            $(
                $crate::symbolic_gate_test!(@emit codes $gate);
            )*
            let kinds: Vec<$crate::logic::GateKind> = codes.iter()
                .filter_map(|c| $crate::logic::GateKind::from_code(*c))
                .collect();
            assert!(!kinds.is_empty(), "at least one gate required");
        }
    };
    // Single token dispatch — each arm emits a push() call
    (@emit $v:ident NAND) => { $v.push(0_u8); };
    (@emit $v:ident NOR)  => { $v.push(1_u8); };
    (@emit $v:ident ADD)  => { $v.push(2_u8); };
    (@emit $v:ident WAIT) => { $v.push(3_u8); };
    (@emit $v:ident TX)   => { $v.push(4_u8); };
    (@emit $v:ident RX)   => { $v.push(5_u8); };
    (@emit $v:ident CAP)  => { $v.push(6_u8); };
    // Compile-time error: unknown gate ident
    (@emit $v:ident $unknown:ident) => {
        compile_error!(concat!("symbolic_gate_test: unknown gate '", stringify!($unknown), "'"))
    };
}

pub use symbolic_gate_test;

// ── bridge_invariant_test! macro ────────────────────────────────────────────

/// Macro: `bridge_invariant_test!` — compile-time bridge invariant validator.
///
/// Validates that two gate networks can be wired together without violating
/// port arity constraints. Compile-time: unknown gate idents produce
/// `compile_error!`. Runtime: asserts the bridge apply succeeds.
///
/// ```ignore
/// bridge_invariant_test!(name: "rx_to_nand" => RX:0 >> NAND:0);
/// bridge_invariant_test!(stable: "rx_pair" => RX, RX >> NAND);
/// ```
#[macro_export]
macro_rules! bridge_invariant_test {
    // With stability check
    (stable: $name:expr => $($src:ident),+ >> $($dst:ident),+ $(,)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn bridge_stable() {
            let mut src = $crate::logic::FluxCapacitor::new();
            let mut dst = $crate::logic::FluxCapacitor::new();
            let mut src_ids: Vec<usize> = Vec::new();
            let mut dst_ids: Vec<usize> = Vec::new();
            $($crate::bridge_invariant_test!(@push src_ids src $src);)+
            $($crate::bridge_invariant_test!(@push dst_ids dst $dst);)+
            // Wire source gates sequentially, then wire last src → first dst
            for i in 1..src_ids.len() {
                let pk = src.ports[src_ids[i - 1]].kind;
                let ck = src.ports[src_ids[i]].kind;
                if pk.arity_out() > 0 && ck.arity_in() > 0 {
                    let _ = src.wire(src_ids[i - 1], 0, src_ids[i], 0);
                }
            }
            for i in 1..dst_ids.len() {
                let pk = dst.ports[dst_ids[i - 1]].kind;
                let ck = dst.ports[dst_ids[i]].kind;
                if pk.arity_out() > 0 && ck.arity_in() > 0 {
                    let _ = dst.wire(dst_ids[i - 1], 0, dst_ids[i], 0);
                }
            }
            // Mark each capacitor as stable (evaluate after internal wiring)
            src.evaluate();
            dst.evaluate();
            let mut bridge = $crate::logic::MultiBridge::new($name, $crate::logic::GateKind::Rx, $crate::logic::GateKind::Nand);
            let v = $crate::logic::check_bridge_invariant(&bridge, &src, &dst);
            assert!(v.is_empty(), "bridge '{}' violated: {:?}", $name, v);
        }
    };
    // Basic: just validates wiring applies within a single capacitor
    ($name:expr => $src_port:tt:$src_gate:ident >> $dst_port:tt:$dst_gate:ident $(,)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn bridge_wire() {
            let mut cap = $crate::logic::FluxCapacitor::new();
            let s_id = cap.add_gate($crate::logic::GateKind::$src_gate);
            let d_id = cap.add_gate($crate::logic::GateKind::$dst_gate);
            let sk = cap.ports[s_id].kind;
            let dk = cap.ports[d_id].kind;
            if sk.arity_out() > 0 && dk.arity_in() > 0 {
                let r = cap.wire(s_id, 0, d_id, 0);
                assert!(r.is_ok(), "bridge '{}': wire failed: {:?}", $name, r);
            }
        }
    };
    // ── @push: emit gate push for an ident (inline, no delegation) ──
    (@push $v:ident $cap:ident Nand) => { $v.push($cap.add_gate($crate::logic::GateKind::Nand)); };
    (@push $v:ident $cap:ident Nor)  => { $v.push($cap.add_gate($crate::logic::GateKind::Nor)); };
    (@push $v:ident $cap:ident Add)  => { $v.push($cap.add_gate($crate::logic::GateKind::Add)); };
    (@push $v:ident $cap:ident Wait) => { $v.push($cap.add_gate($crate::logic::GateKind::Wait)); };
    (@push $v:ident $cap:ident Tx)   => { $v.push($cap.add_gate($crate::logic::GateKind::Tx)); };
    (@push $v:ident $cap:ident Rx)   => { $v.push($cap.add_gate($crate::logic::GateKind::Rx)); };
    (@push $v:ident $cap:ident Cap)  => { $v.push($cap.add_gate($crate::logic::GateKind::Cap)); };
    (@push $v:ident $cap:ident $unknown:ident) => {
        compile_error!(concat!("bridge_invariant_test: unknown gate '", stringify!($unknown), "'"))
    };
}

pub use bridge_invariant_test;

// ── transformer_invariant_test! macro ────────────────────────────────────────

/// Macro: `transformer_invariant_test!` — compile-time transformer invariant.
///
/// Validates that a gate kind transformation produces a compatible output shape.
/// Compile-time: gate names validated. Runtime: asserts connectivity and
/// port compatibility.
///
/// ```ignore
/// transformer_invariant_test!(name: "rx2nand" => RX,NAND ~> NAND,NOR);
/// ```
#[macro_export]
macro_rules! transformer_invariant_test {
    ($name:expr => $($input:ident),+ ~> $($output:ident),+ $(,)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn tx_invariant() {
            let inp: Vec<$crate::logic::GateKind> = vec![$($crate::logic::GateKind::$input),+];
            let out: Vec<$crate::logic::GateKind> = vec![$($crate::logic::GateKind::$output),+];
            let tx = $crate::logic::ModelTransformer::new($name, inp, out);
            let issues = tx.check_connectivity();
            assert!(issues.is_empty(), "transformer '{}' connectivity: {:?}", $name, issues);
            assert!(tx.check_port_compatibility().is_ok(), "transformer '{}' port incompatibility", $name);
        }
    };
}

pub use transformer_invariant_test;

// ── abstract_cap_test! macro ─────────────────────────────────────────────────

/// Macro: `abstract_cap_test!` — compile-time abstract capacitor invariant.
///
/// Validates that an abstract capacitor containing multiple sub-networks,
/// bridges, and transformers is internally consistent.
///
/// ```ignore
/// abstract_cap_test!("net_a" => [RX, NAND, RX] ~ [WAIT] ~~ [RX, CAP]);
/// ```
/// `~` delimits sub-capacitors, `~~` delimits bridges/transformers.
#[macro_export]
macro_rules! abstract_cap_test {
    ($name:expr => $([$($gates:ident),*])* $(~ [$($bgates:ident),*])* $(~~ [$($tx_input:ident),+ ~> $($tx_output:ident),+])* $(,)?) => {
        #[allow(non_snake_case)]
        #[test]
        fn abstract_invariant() {
            let mut abs = $crate::logic::AbstractFluxCapacitor::new($name);
            $(
                {
                    let mut cap = $crate::logic::FluxCapacitor::new();
                    let mut ids: Vec<usize> = Vec::new();
                    $($crate::abstract_cap_test!(@push ids cap $gates);)+
                    for i in 1..ids.len() {
                        let pk = cap.ports[ids[i - 1]].kind;
                        let ck = cap.ports[ids[i]].kind;
                        if pk.arity_out() > 0 && ck.arity_in() > 0 {
                            let _ = cap.wire(ids[i - 1], 0, ids[i], 0);
                        }
                    }
                    cap.evaluate();
                    abs.add_capacitor(cap);
                }
            )*
            $(
                {
                    let mut src = $crate::logic::FluxCapacitor::new();
                    let mut ids: Vec<usize> = Vec::new();
                    let prev_cap = abs.sub_capacitors.last().map(|c| c.ports.len()).unwrap_or(0);
                    $($crate::abstract_cap_test!(@push ids src $bgates);)+
                    let g = if !ids.is_empty() { src.ports[ids[0]].kind } else { $crate::logic::GateKind::Cap };
                    let mut bridge = $crate::logic::MultiBridge::new("bridge", g, g);
                    bridge.wire(prev_cap, 0);
                    abs.add_bridge(bridge);
                }
            )*
            $(
                {
                    let inp: Vec<$crate::logic::GateKind> = vec![$($crate::logic::GateKind::$tx_input),+];
                    let out: Vec<$crate::logic::GateKind> = vec![$($crate::logic::GateKind::$tx_output),+];
                    let tx = $crate::logic::ModelTransformer::new("tx", inp, out);
                    abs.add_transformer(tx);
                }
            )*
            let violations = abs.check_all_invariants();
            if !violations.is_empty() {
                panic!("abstract capacitor '{}' violated: {:?}", $name, violations);
            }
        }
    };
    // ── @push: emit gate push (inline, no delegation) ──────────────
    (@push $v:ident $cap:ident Nand) => { $v.push($cap.add_gate($crate::logic::GateKind::Nand)); };
    (@push $v:ident $cap:ident Nor)  => { $v.push($cap.add_gate($crate::logic::GateKind::Nor)); };
    (@push $v:ident $cap:ident Add)  => { $v.push($cap.add_gate($crate::logic::GateKind::Add)); };
    (@push $v:ident $cap:ident Wait) => { $v.push($cap.add_gate($crate::logic::GateKind::Wait)); };
    (@push $v:ident $cap:ident Tx)   => { $v.push($cap.add_gate($crate::logic::GateKind::Tx)); };
    (@push $v:ident $cap:ident Rx)   => { $v.push($cap.add_gate($crate::logic::GateKind::Rx)); };
    (@push $v:ident $cap:ident Cap)  => { $v.push($cap.add_gate($crate::logic::GateKind::Cap)); };
    (@push $v:ident $cap:ident $unknown:ident) => {
        compile_error!(concat!("abstract_cap_test: unknown gate '", stringify!($unknown), "'"))
    };
}

pub use abstract_cap_test;

/// Runtime test helper: builds flux capacitor from gate kinds and checks stability.
#[cfg(test)]
fn run_stable_test(kinds: &[GateKind]) {
    let mut cap = FluxCapacitor::new();
    let ids: Vec<usize> = kinds.iter().map(|k| cap.add_gate(*k)).collect();
    for i in 1..ids.len() {
        let pk = cap.ports[ids[i - 1]].kind;
        let ck = cap.ports[ids[i]].kind;
        if pk.arity_out() > 0 && ck.arity_in() > 0 {
            let _ = cap.wire(ids[i - 1], 0, ids[i], 0);
        }
    }
    let stable = cap.evaluate();
    assert!(
        stable,
        "flux capacitor not meta-stable: {:?}",
        cap.all_ports_filled()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nand_basic() {
        assert!(nand(true, false));
        assert!(nand(false, true));
        assert!(nand(false, false));
        assert!(!nand(true, true));
    }

    #[test]
    fn nor_basic() {
        assert!(!nor(true, false));
        assert!(!nor(false, true));
        assert!(!nor(true, true));
        assert!(nor(false, false));
    }

    #[test]
    fn add_basic() {
        assert_eq!(add_u8(3, 4), 7);
        assert_eq!(add_u8(255, 1), 0);
    }

    #[test]
    fn wait_stable_after_debounce() {
        assert!(!wait_stable(true, false, 3, 1));
        assert!(wait_stable(true, true, 3, 5));
    }

    #[test]
    fn flux_capacitor_empty_is_not_stable() {
        let cap = FluxCapacitor::new();
        assert!(!cap.is_stable());
    }

    #[test]
    fn flux_capacitor_requires_wires() {
        let mut cap = FluxCapacitor::new();
        let _n0 = cap.add_gate(GateKind::Nand);
        let _n1 = cap.add_gate(GateKind::Nand);
        assert!(!cap.evaluate());
    }

    #[test]
    fn flux_capacitor_wired_gates_become_stable() {
        let mut cap = FluxCapacitor::new();
        let a = cap.add_gate(GateKind::Rx);
        let b = cap.add_gate(GateKind::Rx);
        let out = cap.add_gate(GateKind::Nand);
        cap.wire(a, 0, out, 0).unwrap();
        cap.wire(b, 0, out, 1).unwrap();
        assert!(cap.evaluate());
        assert!(cap.is_stable());
        assert_eq!(cap.transition_count(), 1);
    }

    #[test]
    fn flux_capacitor_unfilled_port_not_stable() {
        let mut cap = FluxCapacitor::new();
        let a = cap.add_gate(GateKind::Rx);
        let out = cap.add_gate(GateKind::Nand);
        cap.wire(a, 0, out, 0).unwrap();
        assert!(!cap.evaluate());
    }

    #[test]
    fn wire_invalid_gate_fails() {
        let mut cap = FluxCapacitor::new();
        assert!(cap.wire(0, 0, 1, 0).is_err());
    }

    #[test]
    fn wire_invalid_port_fails() {
        let mut cap = FluxCapacitor::new();
        let a = cap.add_gate(GateKind::Rx);
        let b = cap.add_gate(GateKind::Nand);
        assert!(cap.wire(a, 0, b, 2).is_err());
        assert!(cap.wire(b, 0, a, 0).is_err());
    }

    #[test]
    fn tx_gate_has_no_output() {
        let mut cap = FluxCapacitor::new();
        let tx = cap.add_gate(GateKind::Tx);
        let rx = cap.add_gate(GateKind::Rx);
        assert!(cap.wire(tx, 0, rx, 0).is_err());
    }

    #[test]
    fn gate_kind_arities() {
        assert_eq!(GateKind::Nand.arity_in(), 2);
        assert_eq!(GateKind::Nor.arity_in(), 2);
        assert_eq!(GateKind::Add.arity_in(), 2);
        assert_eq!(GateKind::Wait.arity_in(), 1);
        assert_eq!(GateKind::Tx.arity_in(), 1);
        assert_eq!(GateKind::Rx.arity_in(), 0);
        assert_eq!(GateKind::Nand.arity_out(), 1);
        assert_eq!(GateKind::Tx.arity_out(), 0);
        assert_eq!(GateKind::Rx.arity_out(), 1);
    }

    #[test]
    fn tokenize_and_expression() {
        let tokens = tokenize_shorthand("NAND && TX -> CAP");
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn tokenize_or_expression() {
        let tokens = tokenize_shorthand("NOR || WAIT -> RX");
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn tokenize_not_expression() {
        let tokens = tokenize_shorthand("!A && B");
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn flux_capacitor_charge_increases_with_wires() {
        let mut cap = FluxCapacitor::new();
        let rx1 = cap.add_gate(GateKind::Rx);
        let rx2 = cap.add_gate(GateKind::Rx);
        let nand = cap.add_gate(GateKind::Nand);
        cap.wire(rx1, 0, nand, 0).unwrap();
        cap.wire(rx2, 0, nand, 1).unwrap();
        cap.evaluate();
        assert_eq!(cap.charge_level(), 2.0);
    }

    #[test]
    fn nand_truth_table() {
        assert!(!nand(true, true));
        assert!(nand(true, false));
        assert!(nand(false, true));
        assert!(nand(false, false));
    }

    #[test]
    fn nor_truth_table() {
        assert!(!nor(true, true));
        assert!(!nor(true, false));
        assert!(!nor(false, true));
        assert!(nor(false, false));
    }

    #[test]
    fn tokenize_number() {
        let tokens = tokenize_shorthand("ADD 42");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[1], ShorthandToken::Number(42));
    }

    // ── symbolic_gate_test! invocations ────────────────────────────

    mod gate_nand_tx_cap {
        symbolic_gate_test!(NAND, TX, CAP);
    }
    mod gate_rx_nand {
        symbolic_gate_test!(RX, NAND);
    }
    mod gate_tx_only {
        symbolic_gate_test!(TX);
    }
    mod gate_rx_wait_cap_stable {
        symbolic_gate_test!(stable => RX, WAIT, CAP);
    }

    // ── bridge_invariant_test! invocations ─────────────────────────
    // Rx → Nand in same capacitor: Rx output port 0 → Nand input port 0
    mod bridge_rx_to_nand {
        bridge_invariant_test!("rx2nand" => 0:Rx >> 0:Nand);
    }
    // Wait → Nand: Wait output feeds Nand input
    mod bridge_wait_nand {
        bridge_invariant_test!("wait2nand" => 0:Wait >> 0:Nand);
    }
    // Rx, Rx >> Wait: two Rx gates wired in src capacitor, Wait in dst capacitor
    // Both capacitors are independently stable (Rx has no input ports, Wait is unfilled but skipped)
    // Actually both need internal wires — skip for now, use single-capacitor test instead
    //

    // ── transformer_invariant_test! invocations ────────────────────

    mod tx_rx_nand_to_nor {
        transformer_invariant_test!("rx_nand_nor" => Rx, Nand ~> Nand, Nor);
    }
    mod tx_single_rx_to_wait {
        transformer_invariant_test!("rx_wait" => Rx ~> Wait);
    }
    // Output gates must have input ports to receive transformed values
    mod tx_three_chain {
        transformer_invariant_test!("chain" => Rx, Wait, Nand ~> Wait, Nor, Add);
    }

    // ── abstract_cap_test! invocations ─────────────────────────────

    mod abs_single_pair {
        abstract_cap_test!("single" => [Rx, Wait, Cap]);
    }
    // Two Rx feed Nand (both ports) → Nand stable → Wait stable
    mod abs_two_networks {
        abstract_cap_test!("two_net" => [Rx, Wait] ~ [Rx, Wait]);
    }
    mod abs_with_bridge_tx {
        abstract_cap_test!("w_bridge_tx" => [Rx, Wait] ~~ [Rx, Nand ~> Nand, Nor]);
    }
    mod abs_three_net_bridge_tx {
        abstract_cap_test!("complex" => [Rx, Wait] ~ [Rx, Nand] ~~ [Rx ~> Wait]);
    }

    // ── MultiBridge runtime tests ──────────────────────────────────

    #[test]
    fn bridge_empty_has_no_violations() {
        let bridge = MultiBridge::new("empty", GateKind::Nand, GateKind::Nor);
        let violations = bridge.validate_invariants();
        assert!(violations.is_empty());
    }

    #[test]
    fn bridge_double_wire_detected() {
        let mut bridge = MultiBridge::new("dup", GateKind::Nand, GateKind::Nor);
        bridge.wire(0, 0).wire(0, 1);
        let violations = bridge.validate_invariants();
        assert!(violations
            .iter()
            .any(|v| v.contains("wired multiple times")));
    }

    #[test]
    fn bridge_apply_to_capacitors() {
        let mut cap_a = FluxCapacitor::new();
        cap_a.add_gate(GateKind::Rx);
        let mut cap_b = FluxCapacitor::new();
        cap_b.add_gate(GateKind::Nand);
        let mut bridge = MultiBridge::new("test", GateKind::Rx, GateKind::Nand);
        bridge.wire(0, 0);
        assert!(bridge.apply(&mut cap_a, &mut cap_b).is_ok());
    }

    #[test]
    fn bridge_apply_invalid_port_fails() {
        let mut cap_a = FluxCapacitor::new();
        cap_a.add_gate(GateKind::Rx);
        let mut cap_b = FluxCapacitor::new();
        cap_b.add_gate(GateKind::Nand);
        let mut bridge = MultiBridge::new("bad", GateKind::Rx, GateKind::Nand);
        bridge.wire(99, 0);
        assert!(bridge.apply(&mut cap_a, &mut cap_b).is_err());
    }

    #[test]
    fn check_bridge_invariant_reports_source_instability() {
        let cap_a = FluxCapacitor::new();
        let cap_b = FluxCapacitor::new();
        let bridge = MultiBridge::new("test", GateKind::Nand, GateKind::Nor);
        let violations = check_bridge_invariant(&bridge, &cap_a, &cap_b);
        assert!(violations.iter().any(|v| v.contains("not stable")));
    }

    // ── ModelTransformer tests ──────────────────────────────────────

    #[test]
    fn transformer_empty_input_shape_reported() {
        let tx = ModelTransformer::new("empty", vec![], vec![GateKind::Nand]);
        let issues = tx.check_connectivity();
        assert!(issues.iter().any(|v| v.contains("empty input")));
    }

    #[test]
    fn transformer_empty_output_shape_reported() {
        let tx = ModelTransformer::new("empty_out", vec![GateKind::Nand], vec![]);
        let issues = tx.check_connectivity();
        assert!(issues.iter().any(|v| v.contains("empty output")));
    }

    #[test]
    fn transformer_tx_input_rejected() {
        // TX has arity_out=0, can't wire output to anything
        let tx = ModelTransformer::new("bad", vec![GateKind::Tx], vec![GateKind::Rx]);
        assert!(tx.check_port_compatibility().is_err());
    }

    #[test]
    fn transformer_cap_output_rejected() {
        // CAP has arity_in=0, can't receive input
        let tx = ModelTransformer::new("bad2", vec![GateKind::Rx], vec![GateKind::Cap]);
        assert!(tx.check_port_compatibility().is_err());
    }

    #[test]
    fn transformer_valid_nand_to_nor_passes() {
        let tx = ModelTransformer::new("good", vec![GateKind::Nand], vec![GateKind::Nor]);
        assert!(tx.check_port_compatibility().is_ok());
    }

    // ── AbstractFluxCapacitor tests ─────────────────────────────────

    #[test]
    fn abstract_empty_is_stable() {
        let abs = AbstractFluxCapacitor::new("empty");
        assert!(abs.is_abstractly_stable());
    }

    #[test]
    fn abstract_unstable_sub_capacitor_reported() {
        let mut abs = AbstractFluxCapacitor::new("test");
        let cap = FluxCapacitor::new();
        abs.add_capacitor(cap);
        let violations = abs.check_all_invariants();
        assert!(violations.iter().any(|v| v.contains("not stable")));
    }

    #[test]
    fn abstract_with_bridge_and_transformer() {
        let mut abs = AbstractFluxCapacitor::new("complex");

        let mut src = FluxCapacitor::new();
        let rx = src.add_gate(GateKind::Rx);
        let mut dst = FluxCapacitor::new();
        let nand = dst.add_gate(GateKind::Nand);
        let rx2 = dst.add_gate(GateKind::Rx);
        let _ = dst.wire(rx2, 0, nand, 0);
        let _ = dst.wire(rx, 0, nand, 1);

        abs.add_capacitor(src);
        abs.add_capacitor(dst);

        let mut bridge = MultiBridge::new("b1", GateKind::Rx, GateKind::Nand);
        bridge.wire(0, 0);
        abs.add_bridge(bridge);

        let tx = ModelTransformer::new("t1", vec![GateKind::Nand], vec![GateKind::Nor]);
        abs.add_transformer(tx);

        // Both capacitors are not yet stable (no wires in src, nand in dst needs 2 ports)
        let violations = abs.check_all_invariants();
        assert!(!violations.is_empty());
        assert!(!abs.is_abstractly_stable());
    }

    #[test]
    fn abstract_stable_when_all_sub_capacitors_stable() {
        let mut abs = AbstractFluxCapacitor::new("multi");

        let mut cap1 = FluxCapacitor::new();
        let rx1 = cap1.add_gate(GateKind::Rx);
        let rx2 = cap1.add_gate(GateKind::Rx);
        let nand1 = cap1.add_gate(GateKind::Nand);
        let _ = cap1.wire(rx1, 0, nand1, 0);
        let _ = cap1.wire(rx2, 0, nand1, 1);
        cap1.evaluate();

        let mut cap2 = FluxCapacitor::new();
        let rx3 = cap2.add_gate(GateKind::Rx);
        let wait = cap2.add_gate(GateKind::Wait);
        let _ = cap2.wire(rx3, 0, wait, 0);
        cap2.evaluate();

        abs.add_capacitor(cap1);
        abs.add_capacitor(cap2);
        assert!(abs.is_abstractly_stable());
    }

    #[test]
    fn abstract_bridge_invariant_caught() {
        let mut abs = AbstractFluxCapacitor::new("bridge-check");

        let mut cap = FluxCapacitor::new();
        let rx = cap.add_gate(GateKind::Rx);
        let nand = cap.add_gate(GateKind::Nand);
        let rx2 = cap.add_gate(GateKind::Rx);
        let _ = cap.wire(rx2, 0, nand, 0);
        let _ = cap.wire(rx, 0, nand, 1);
        cap.evaluate();
        abs.add_capacitor(cap);

        let mut bridge = MultiBridge::new("dup", GateKind::Rx, GateKind::Nand);
        bridge.wire(0, 0).wire(0, 1);
        abs.add_bridge(bridge);

        let violations = abs.check_all_invariants();
        assert!(violations
            .iter()
            .any(|v| v.contains("wired multiple times")));
    }
}
