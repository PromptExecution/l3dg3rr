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
    (0, "NAND"), (1, "NOR"), (2, "ADD"), (3, "WAIT"),
    (4, "TX"),   (5, "RX"),  (6, "CAP"),
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

    pub fn wire(&mut self, from_gate: usize, from_port: usize, to_gate: usize, to_port: usize) -> Result<(), String> {
        if from_gate >= self.ports.len() {
            return Err(format!("from_gate {from_gate} does not exist"));
        }
        if to_gate >= self.ports.len() {
            return Err(format!("to_gate {to_gate} does not exist"));
        }
        let from_kind = self.ports[from_gate].kind;
        if from_port >= from_kind.arity_out() {
            return Err(format!("from_gate {from_gate} ({from_kind:?}) has no output port {from_port}"));
        }
        let to_kind = self.ports[to_gate].kind;
        if to_port >= to_kind.arity_in() {
            return Err(format!("to_gate {to_gate} ({to_kind:?}) has no input port {to_port}"));
        }
        let wire = Wire { from_gate, from_port, to_gate, to_port };
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
                unfilled.push((gate.id, format!("{kind:?} gate {} has {filled}/{required} ports filled", gate.id)));
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
            '&' if chars.peek() == Some(&'&') => { chars.next(); tokens.push(ShorthandToken::And); }
            '|' if chars.peek() == Some(&'|') => { chars.next(); tokens.push(ShorthandToken::Or); }
            '!' => tokens.push(ShorthandToken::Not),
            '-' if chars.peek() == Some(&'>') => { chars.next(); tokens.push(ShorthandToken::Arrow); }
            '<' if chars.peek() == Some(&'-') => { chars.next(); tokens.push(ShorthandToken::BackArrow); }
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
                    if c.is_ascii_alphanumeric() || c == '_' || c == '.' { s.push(c); chars.next(); } else { break; }
                }
                tokens.push(ShorthandToken::Ident(s));
            }
            _ => {}
        }
    }
    tokens
}

// ── Gate evaluation functions ────────────────────────────────────────────────

pub fn nand(a: bool, b: bool) -> bool { !(a && b) }
pub fn nor(a: bool, b: bool) -> bool { !(a || b) }
pub fn add_u8(a: u8, b: u8) -> u8 { a.wrapping_add(b) }
pub fn wait_stable(current: bool, previous: bool, debounce_ticks: u64, tick: u64) -> bool {
    if current == previous && tick >= debounce_ticks { current } else { previous }
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

/// Runtime test helper: builds flux capacitor from gate kinds and checks stability.
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
    assert!(stable, "flux capacitor not meta-stable: {:?}", cap.all_ports_filled());
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
    // Each generates a test that validates gate names at compile time.

    mod gate_nand_tx_cap { symbolic_gate_test!(NAND, TX, CAP); }
    mod gate_rx_nand { symbolic_gate_test!(RX, NAND); }
    mod gate_tx_only { symbolic_gate_test!(TX); }
    mod gate_rx_wait_cap_stable { symbolic_gate_test!(stable => RX, WAIT, CAP); }
    mod gate_rx_nand_stable { symbolic_gate_test!(stable => RX, NAND, RX); }
}
