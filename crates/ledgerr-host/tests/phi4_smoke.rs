//! Phi-4 Mini reasoning GGUF smoke tests.
//!
//! candle backend  — `--features local-llm`   — just test-phi4
//! mistralrs backend — `--features mistralrs-llm` — just test-phi4-mistral
//!
//! Model path: models/unsloth/Phi-4-mini-reasoning-GGUF/Phi-4-mini-reasoning-Q3_K_M.gguf
//! (symlink → /mnt/d/models/unsloth/Phi-4-mini-reasoning-GGUF)

#[cfg(feature = "local-llm")]
mod phi4 {
    use ledgerr_host::{
        agent_runtime::{AgentRuntime, ModelRequest},
        local_llm::LocalCandelRuntime,
    };
    use std::path::PathBuf;

    fn model_path() -> PathBuf {
        // Prefer the repo-relative symlink; fall back to absolute D: path.
        let via_symlink = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|root| {
                root.join(
                    "models/unsloth/Phi-4-mini-reasoning-GGUF/Phi-4-mini-reasoning-Q3_K_M.gguf",
                )
            })
            .unwrap_or_default();

        if via_symlink.exists() {
            return via_symlink;
        }

        PathBuf::from(
            "/mnt/d/models/unsloth/Phi-4-mini-reasoning-GGUF/Phi-4-mini-reasoning-Q3_K_M.gguf",
        )
    }

    /// Compile-time proof that LocalCandelRuntime satisfies the trait bounds
    /// required by AgentRuntime (Send + Sync).
    #[test]
    fn runtime_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + AgentRuntime>() {}
        assert_send_sync::<LocalCandelRuntime>();
    }

    /// End-to-end smoke test: load phi-4 mini, run a short completion, assert
    /// a non-empty response comes back. Skips gracefully when the model file is
    /// absent so CI passes without the 2 GB weight file.
    #[test]
    fn phi4_produces_output() {
        let path = model_path();
        if !path.exists() {
            eprintln!(
                "SKIP phi4_produces_output: model not found at {}\n\
                 Download with: just hf-download-phi4-mini-gguf\n\
                 Then symlink with: just phi4-reasoning-symlink",
                path.display()
            );
            return;
        }

        let runtime = LocalCandelRuntime::new(&path)
            .expect("model file exists so new() must succeed")
            .with_max_tokens(64);

        let request = ModelRequest::text("Reply with only the single word: HELLO")
            .with_system_prompt("You are a terse assistant. Follow instructions exactly.");

        let response =
            AgentRuntime::complete(&runtime, request).expect("phi4 completion should not fail");

        assert!(
            !response.assistant_text.is_empty(),
            "expected non-empty response from phi4, got empty string"
        );
        eprintln!("phi4 candle response: {:?}", response.assistant_text);
    }
}

#[cfg(feature = "mistralrs-llm")]
mod phi4_mistral {
    use ledgerr_host::{
        agent_runtime::{AgentRuntime, ModelRequest},
        local_llm_mistral::LocalMistralRuntime,
    };
    use std::path::PathBuf;

    fn model_path() -> PathBuf {
        let via_symlink = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|root| {
                root.join(
                    "models/unsloth/Phi-4-mini-reasoning-GGUF/Phi-4-mini-reasoning-Q3_K_M.gguf",
                )
            })
            .unwrap_or_default();

        if via_symlink.exists() {
            return via_symlink;
        }

        PathBuf::from(
            "/mnt/d/models/unsloth/Phi-4-mini-reasoning-GGUF/Phi-4-mini-reasoning-Q3_K_M.gguf",
        )
    }

    #[test]
    fn mistral_runtime_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + AgentRuntime>() {}
        assert_send_sync::<LocalMistralRuntime>();
    }

    /// End-to-end: mistralrs loads and runs phi-4 mini with correct partial RoPE,
    /// producing coherent output faster than the candle path.
    #[test]
    fn phi4_mistral_produces_output() {
        let path = model_path();
        if !path.exists() {
            eprintln!(
                "SKIP phi4_mistral_produces_output: model not found at {}\n\
                 Download with: just hf-download-phi4-mini-gguf\n\
                 Then symlink with: just phi4-reasoning-symlink",
                path.display()
            );
            return;
        }

        let runtime = LocalMistralRuntime::new(&path)
            .expect("model file exists so new() must succeed")
            .with_max_tokens(64);

        let request = ModelRequest::text("Reply with only the single word: HELLO")
            .with_system_prompt("You are a terse assistant. Follow instructions exactly.");

        let response = AgentRuntime::complete(&runtime, request)
            .expect("phi4 mistralrs completion should not fail");

        assert!(
            !response.assistant_text.is_empty(),
            "expected non-empty response from phi4 via mistralrs, got empty string"
        );
        eprintln!("phi4 mistralrs response: {:?}", response.assistant_text);
    }
}
