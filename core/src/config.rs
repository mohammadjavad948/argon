use anyhow::Result;

/// Trait for building configuration structs
/// 
/// Users should implement this trait to specify how their config is created.
/// The `#[derive(Config)]` macro will use this implementation to initialize
/// the config lazily using `OnceCell`.
pub trait ConfigBuilder: Clone + Send + Sync + 'static {
    /// Build the configuration instance
    /// 
    /// This method should read from environment variables, files, or other
    /// sources and construct the configuration struct.
    fn build() -> Result<Self>;
}

