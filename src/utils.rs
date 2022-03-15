use eyre::{Result as EResult, WrapErr};
use tracing::*;

const DEFAULT_TRACING_DIRECTIVES: &'static str = "info";

pub(crate) fn env_var(key: &str) -> EResult<String> {
    match std::env::var(key) {
        Ok(value) => {
            trace!(?key, ?value, "got environment variable");
            Ok(value)
        }
        Err(err) => {
            warn!(?key, ?err, "failed to get environment variable");
            Err(err).wrap_err_with(|| format!("Failed to get environment variable {key}"))
        }
    }
}

pub(crate) fn setup_tracing_subscriber() -> EResult<()> {
    use tracing_subscriber::EnvFilter;

    let directives = env_var("RUST_LOG").unwrap_or(DEFAULT_TRACING_DIRECTIVES.to_string());
    let filter = EnvFilter::new(directives);
    let subscriber = tracing_subscriber::fmt()
        //.json()
        .with_thread_names(true)
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .wrap_err("unable to set default global tracing subscriber")?;

    Ok(())
}

pub(crate) fn db_uri() -> EResult<String> {
    let uri = format!(
        "postgres://{user}:{pass}@{host}:{port}/{name}",
        user = env_var("DB_USER")?,
        pass = env_var("DB_PASS")?,
        host = env_var("DB_HOST")?,
        port = env_var("DB_PORT").unwrap_or("5432".into()),
        name = env_var("DB_NAME").unwrap_or("meishu".into()),
    );
    debug!(?uri, "got database connection uri from environment");
    Ok(uri)
}
