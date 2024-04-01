use clap::builder::TypedValueParser;
use courageous_format::Position3d;

#[derive(Clone, Copy, Debug)]
pub struct Position3dParser;

impl TypedValueParser for Position3dParser {
    type Value = Position3d;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let mut components = value
            .to_str()
            .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidUtf8).with_cmd(cmd))?
            .split(',');
        let longitude = components
            .next()
            .ok_or_else(|| {
                let mut err =
                    clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
                err.insert(
                    clap::error::ContextKind::MinValues,
                    clap::error::ContextValue::Number(2),
                );
                err
            })?
            .parse()
            .map_err(|_| {
                let mut err =
                    clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
                err.insert(
                    clap::error::ContextKind::InvalidValue,
                    clap::error::ContextValue::String(
                        "Must be a valid floating point number".to_string(),
                    ),
                );
                err
            })?;
        let latitude = components
            .next()
            .ok_or_else(|| {
                let mut err =
                    clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
                err.insert(
                    clap::error::ContextKind::MinValues,
                    clap::error::ContextValue::Number(2),
                );
                err
            })?
            .parse()
            .map_err(|_| {
                let mut err =
                    clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
                err.insert(
                    clap::error::ContextKind::InvalidValue,
                    clap::error::ContextValue::String(
                        "Must be a valid floating point number".to_string(),
                    ),
                );
                err
            })?;
        let altitude = if let Some(c) = components.next() {
            c.parse().map_err(|_| {
                let mut err =
                    clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);
                err.insert(
                    clap::error::ContextKind::InvalidValue,
                    clap::error::ContextValue::String(
                        "Must be a valid floating point number".to_string(),
                    ),
                );
                err
            })?
        } else {
            0.
        };

        Ok(Position3d {
            lon: longitude,
            lat: latitude,
            height: altitude,
        })
    }
}
