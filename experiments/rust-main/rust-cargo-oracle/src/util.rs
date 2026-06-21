use anyhow::Result;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub(crate) fn generated_timestamp_string() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

#[cfg(test)]
mod tests {
    use super::generated_timestamp_string;
    use anyhow::Result;

    #[test]
    fn generated_timestamp_uses_rfc3339_shape() -> Result<()> {
        let generated = generated_timestamp_string()?;

        assert!(generated.contains('T'), "{generated}");
        assert!(generated.ends_with('Z'), "{generated}");
        assert!(
            generated
                .chars()
                .any(|character| character == '-' || character == ':'),
            "{generated}"
        );
        Ok(())
    }
}
