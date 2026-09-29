use thiserror::Error;

#[derive(Debug, Error)]
pub enum TauriError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl serde::Serialize for TauriError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            TauriError::Anyhow(error) => {
                let trace = error
                    .chain()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\nCaused by: ");
                eprintln!("Tauri command failed with trace back: \n{trace}");
                serializer.serialize_str(&trace)
            }
        }
    }
}
