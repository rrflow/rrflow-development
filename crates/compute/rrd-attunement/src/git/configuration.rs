//! The admitted repository-local subset. No environment, home directory,
//! include expansion or worktree discovery is consulted by this parser.
use super::{Error, ObjectFormat, ParseLimits};

#[derive(Debug)]
pub struct RepositoryConfig {
    pub ignore_case: bool,
    pub object_format: ObjectFormat,
}

impl RepositoryConfig {
    pub fn from_bytes(bytes: &[u8], limits: ParseLimits) -> Result<Self, Error> {
        limits.validate()?;
        if bytes.len() > limits.input_bytes || bytes.len() > limits.allocation_bytes {
            return Err(Error::InputLimit);
        }
        // Bound syntax events by admitted bytes; this is not an RSS assertion.
        let config = gix_config::File::from_bytes_no_includes(
            bytes,
            gix_config::file::Metadata::api(),
            Default::default(),
        )
        .map_err(|_| Error::InvalidConfiguration)?;
        if config
            .sections()
            .map(|s| 1 + s.value_names().count())
            .sum::<usize>()
            > limits.entries
        {
            return Err(Error::EntryLimit);
        }
        if config.sections_by_name("include").is_some()
            || config.sections_by_name("includeIf").is_some()
            || config.string_implicit("core.excludesFile").is_some()
            || config.string_implicit("core.worktree").is_some()
        {
            return Err(Error::UnsupportedConfiguration);
        }
        for section in config.sections_by_name("extensions").into_iter().flatten() {
            if section.header().subsection_name().is_some()
                || section
                    .value_names()
                    .any(|key| !key.eq_ignore_ascii_case("objectFormat"))
            {
                return Err(Error::UnsupportedConfiguration);
            }
        }
        let boolean = |key| config.boolean(key).map_err(|_| Error::InvalidConfiguration);
        if boolean("core.bare")?.unwrap_or(false)
            || boolean("core.sparseCheckout")?.unwrap_or(false)
        {
            return Err(Error::UnsupportedConfiguration);
        }
        let version = config
            .integer("core.repositoryFormatVersion")
            .map_err(|_| Error::InvalidConfiguration)?
            .unwrap_or(0);
        if !(0..=1).contains(&version) {
            return Err(Error::UnsupportedConfiguration);
        }
        if config.string_implicit("extensions.objectFormat") == Some(None) {
            return Err(Error::InvalidConfiguration);
        }
        let object_format = match config
            .string("extensions.objectFormat")
            .as_ref()
            .map(|v| v.as_slice())
        {
            None => ObjectFormat::Sha1,
            Some(b"sha1") if version == 1 => ObjectFormat::Sha1,
            Some(b"sha256") if version == 1 => ObjectFormat::Sha256,
            _ => return Err(Error::UnsupportedConfiguration),
        };
        Ok(Self {
            ignore_case: boolean("core.ignoreCase")?.unwrap_or(false),
            object_format,
        })
    }
}
