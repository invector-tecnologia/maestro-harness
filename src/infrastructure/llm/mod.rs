//! LLM provider adapters (Ollama by default; OpenAI/Anthropic/Gemini optional).
//!
//! Each adapter implements the domain `LlmProvider` port. The Ollama reference
//! adapter is TASK 009; the registry/factory is TASK 008.

pub mod ollama;
pub mod registry;

pub use ollama::OllamaProvider;
pub use registry::{FactoryError, ProviderRegistry};
