use std::{borrow::Cow, error::Error};

use aspeak::{
    get_endpoint_by_region, AspeakError, AudioFormat, AuthOptions, Role, TextOptions,
    DEFAULT_ENDPOINT, DEFAULT_VOICES, QUALITY_MAP,
};
use clap::{ArgAction, Args, ValueEnum};
use color_eyre::eyre::anyhow;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use strum::AsRefStr;

use super::config::{AuthConfig, Config, OutputFormatConfig};

#[derive(Debug, Clone, Copy, Default, ValueEnum, AsRefStr, Deserialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ContainerFormat {
    Mp3,
    Ogg,
    Webm,
    #[default]
    Wav,
}

#[derive(Args, Debug)]
pub struct ProfileArgs {
    #[arg(long, action = ArgAction::SetTrue, help = "Do not use profile")]
    no_profile: bool,
    #[arg(long, conflicts_with = "no_profile", help = "The profile to use")]
    profile: Option<String>,
}

impl ProfileArgs {
    pub(crate) fn load_profile(&self) -> color_eyre::Result<Option<Config>> {
        if self.no_profile {
            Ok(None)
        } else {
            Ok(Config::load(self.profile.as_ref())?)
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct AuthArgs {
    #[arg(short, long, help = "Endpoint of TTS API")]
    pub endpoint: Option<String>,
    #[arg(
        short,
        long,
        help = "If you are using official endpoints, you can specify a region instead of full endpoint url",
        conflicts_with = "endpoint"
    )]
    pub region: Option<String>,
    #[arg(short, long, help = "Auth token for speech service")]
    pub token: Option<String>,
    #[arg(short, long, help = "Speech resource key")]
    pub key: Option<String>,
    #[arg(short = 'H', long,value_parser = parse_header, help = "Additional request headers")]
    pub headers: Vec<(HeaderName, HeaderValue)>,
}

impl AuthArgs {
    pub(crate) fn to_auth_options<'a>(
        &'a self,
        auth_config: Option<&'a AuthConfig>,
    ) -> color_eyre::Result<AuthOptions<'a>> {
        Ok(AuthOptions {
            endpoint: self
                .endpoint
                .as_deref()
                .map(Cow::Borrowed)
                .or_else(|| {
                    self.region
                        .as_deref()
                        .map(get_endpoint_by_region)
                        .map(Cow::Owned)
                })
                .or_else(|| {
                    auth_config
                        .map(|c| c.endpoint.as_ref().map(Cow::from))
                        .flatten()
                })
                .unwrap_or(Cow::Borrowed(DEFAULT_ENDPOINT)),
            token: match (self.token.as_deref(), auth_config) {
                (Some(token), _) => Some(Cow::Borrowed(token)),
                (None, Some(config)) => config.token.as_ref().map(Cow::from),
                (None, None) => None,
            },
            key: match (self.key.as_deref(), auth_config) {
                (Some(key), _) => Some(Cow::Borrowed(key)),
                (None, Some(config)) => config.key.as_ref().map(Cow::from),
                (None, None) => None,
            },
            headers: if let Some(AuthConfig {
                headers: Some(headers),
                ..
            }) = auth_config
            {
                let vec: color_eyre::Result<Vec<(HeaderName, HeaderValue)>> = headers
                    .iter()
                    .map(|(k, v)| {
                        Ok((
                            HeaderName::from_bytes(k.as_bytes())?,
                            HeaderValue::from_bytes(v.as_bytes())?,
                        ))
                    })
                    .collect();
                let mut vec = vec?;
                vec.extend_from_slice(&self.headers);
                Cow::Owned(vec)
            } else {
                Cow::Borrowed(&self.headers)
            },
        })
    }
}

impl<'a> TryInto<AuthOptions<'a>> for &'a AuthArgs {
    type Error = AspeakError;

    fn try_into(self) -> Result<AuthOptions<'a>, Self::Error> {
        Ok(AuthOptions {
            endpoint: self
                .endpoint
                .as_deref()
                .map(Cow::Borrowed)
                .or_else(|| {
                    self.region
                        .as_deref()
                        .map(get_endpoint_by_region)
                        .map(Cow::Owned)
                })
                .unwrap_or(Cow::Borrowed(DEFAULT_ENDPOINT)),
            token: self.token.as_deref().map(Cow::Borrowed),
            key: self.key.as_deref().map(Cow::Borrowed),
            headers: Cow::Borrowed(&self.headers),
        })
    }
}

#[derive(Args, Debug, Default)]
pub(crate) struct InputArgs {
    #[arg(short, long, help = "Text/SSML file to speak, default to `-`(stdin)")]
    pub file: Option<String>,
    #[arg(short, long, help = "Text/SSML file encoding")]
    pub encoding: Option<String>,
}

#[derive(Args, Debug, Default)]
pub(crate) struct OutputArgs {
    #[arg(short, long, help = "Output file path")]
    pub output: Option<String>,
    #[arg(
        short,
        long,
        allow_negative_numbers = true,
        help = "Output quality, default to 0. Run `aspeak list-qualities` to list available quality levels"
    )]
    pub quality: Option<i32>,
    #[arg(short, long)]
    pub container_format: Option<ContainerFormat>,
    #[arg(
        short = 'F',
        long,
        conflicts_with = "quality",
        conflicts_with = "container_format",
        hide_possible_values = true,
        help = "Set output audio format (experts only). Run `aspeak list-formats` to list available formats"
    )]
    pub format: Option<AudioFormat>,
}

impl OutputArgs {
    pub(crate) fn get_audio_format(
        &self,
        config: Option<&OutputFormatConfig>,
    ) -> color_eyre::Result<AudioFormat> {
        Ok(
            match (self.format, self.container_format, self.quality, config) {
                (Some(format), _, _, _) => format,
                (_, Some(container), quality, _) => QUALITY_MAP
                    .get(container.as_ref())
                    .unwrap()
                    .get(&(quality.unwrap_or_default() as i8))
                    .map(|x| *x)
                    .ok_or_else(|| {
                        anyhow!(format!(
                            "Invalid quality {:?} for container type {}",
                            quality,
                            container.as_ref()
                        ))
                    })?,
                (_, _, Some(_quality), _) => {
                    todo!()
                }
                (_, _, _, Some(OutputFormatConfig::AudioFormat { format })) => *format,
                (_, _, _, Some(OutputFormatConfig::ContaierAndQuality { container, quality })) => {
                    QUALITY_MAP
                        .get(container.unwrap_or_default().as_ref())
                        .unwrap()
                        .get(&(quality.unwrap_or_default() as i8))
                        .map(|x| *x)
                        .ok_or_else(|| {
                            anyhow!(format!(
                                "Invalid quality {:?} for container type {:?}",
                                quality, container
                            ))
                        })?
                }
                (None, None, None, None) => Default::default(),
            },
        )
    }
}

#[derive(Args, Debug, Default)]
pub(crate) struct TextArgs {
    #[clap(help = "The text to speak. \
                If neither text nor input file is specified, the text will be read from stdin.")]
    pub text: Option<String>,
    #[arg(short, long, value_parser = parse_pitch,
        help="Set pitch, default to 0. \
              Valid values include floats(will be converted to percentages), \
              percentages such as 20% and -10%, absolute values like 300Hz, \
              and relative values like -20Hz, +2st and string values like x-low. \
              See the documentation for more details.")]
    pub pitch: Option<String>,
    #[arg(short, long, value_parser = parse_rate,
        help=r#"Set speech rate, default to 0. \
                Valid values include floats(will be converted to percentages), \
                percentages like -20%%, floats with postfix "f" \
                (e.g. 2f means doubling the default speech rate), \
                and string values like x-slow. See the documentation for more details."# )]
    pub rate: Option<String>,
    #[arg(short = 'S', long, help = r#"Set speech style, default to "general""#)]
    pub style: Option<String>,
    #[arg(short = 'R', long)]
    pub role: Option<Role>,
    #[arg(
        short = 'd',
        long,
        value_parser = parse_style_degree,
        help = "Specifies the intensity of the speaking style. This only works for some Chinese voices!"
    )]
    pub style_degree: Option<f32>,
    #[arg(short, long, conflicts_with = "locale", help = "Voice to use")]
    pub voice: Option<String>,
    #[arg(short, long, help = "Locale to use, default to en-US")]
    pub locale: Option<String>,
}

impl<'a> TryInto<TextOptions<'a>> for &'a TextArgs {
    type Error = AspeakError;

    fn try_into(self) -> Result<TextOptions<'a>, Self::Error> {
        Ok(TextOptions {
            text: self.text.as_deref().unwrap(),
            voice: self
                .voice
                .as_deref()
                .or_else(|| {
                    DEFAULT_VOICES
                        .get(self.locale.as_deref().unwrap_or("en-US"))
                        .map(|x| *x)
                })
                .unwrap(),
            pitch: self.pitch.as_deref(),
            rate: self.rate.as_deref(),
            style: self.style.as_deref(),
            role: self.role,
            style_degree: self.style_degree,
        })
    }
}

/// Parse a single key-value pair
fn parse_header(
    s: &str,
) -> Result<(HeaderName, HeaderValue), Box<dyn Error + Send + Sync + 'static>> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((
        HeaderName::from_bytes(s[..pos].as_bytes())?,
        HeaderValue::from_str(&s[pos + 1..])?,
    ))
}

fn is_float(s: &str) -> bool {
    return s.parse::<f32>().is_ok();
}

pub(crate) fn parse_pitch(arg: &str) -> Result<String, String> {
    if (arg.ends_with("Hz") && is_float(&arg[..arg.len() - 2]))
        || (arg.ends_with("%") && is_float(&arg[..arg.len() - 1]))
        || (arg.ends_with("st")
            && (arg.starts_with('+') || arg.starts_with('-'))
            && is_float(&arg[..arg.len() - 2]))
        || ["default", "x-low", "low", "medium", "high", "x-high"].contains(&arg)
    {
        Ok(arg.to_owned())
    } else if let Ok(v) = arg.parse::<f32>() {
        // float values that will be converted to percentages
        Ok(format!("{:.2}", v * 100f32))
    } else {
        Err(format!(
            "Please read the documentation for possible values of pitch."
        ))
    }
}

pub(crate) fn parse_rate(arg: &str) -> Result<String, String> {
    if (arg.ends_with("%") && is_float(&arg[..arg.len() - 1]))
        || ["default", "x-slow", "slow", "medium", "fast", "x-fast"].contains(&arg)
    {
        Ok(arg.to_owned())
    } else if arg.ends_with('f') && is_float(&arg[..arg.len() - 1]) {
        // raw float
        Ok(arg[..arg.len() - 1].to_owned())
    } else if let Ok(v) = arg.parse::<f32>() {
        // float values that will be converted to percentages
        Ok(format!("{:.2}", v * 100f32))
    } else {
        Err(format!(
            "Please read the documentation for possible values of rate."
        ))
    }
}

fn parse_style_degree(arg: &str) -> Result<f32, String> {
    if let Ok(v) = arg.parse::<f32>() {
        if validate_style_degree(v) {
            Ok(v)
        } else {
            Err(format!("Value {v} out of range [0.01, 2]"))
        }
    } else {
        Err("Not a floating point number!".to_owned())
    }
}

pub(crate) fn validate_style_degree(degree: f32) -> bool {
    0.01f32 <= degree && degree <= 2.0f32
}
