//! A simple text-to-speech client for Azure TTS API.
//!
//! This crate provides the client binding for Azure TTS API. It supports both RESTful API and Websocket API.
//!
//! # Quick Start
//!
//! First, you need to setup authentication and select an audio format.
//! Here we will use an Azure subscription key, but you can also use an auth token.
//!
//! ```ignore
//! use aspeak::{synthesizer::SynthesizerConfig, AudioFormat, AuthOptionsBuilder};
//! use aspeak::{get_rest_endpoint_by_region, get_websocket_endpoint_by_region};
//!
//! let auth = AuthOptionsBuilder::new(
//!     // Choose one of the following endpoints based on your selected API.
//!     // get_rest_endpoint_by_region("eastus"), // for RESTful API
//!     // get_websocket_endpoint_by_region("eastus") // for Websocket API
//! )
//! .key("YOUR_AZURE_SUBSCRIPTION_KEY")
//! .build();
//! let config = SynthesizerConfig::new(auth, AudioFormat::Riff16Khz16BitMonoPcm);
//! ```
//!
//! ## RESTful Synthesizer
//!
//! Then, you can create a [RestSynthesizer][crate::synthesizer::RestSynthesizer]
//! from the [SynthesizerConfig][crate::synthesizer::SynthesizerConfig].
//!
//! ```ignore
//! let rest_syn = config.rest_synthesizer()?;
//! ```
//!
//! Now you can synthesize SSML to audio data.
//!
//! ```ignore
//! let ssml = r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US"><voice name="en-US-JennyNeural">Hello, world!</voice></speak>"#;
//! let audio_data = rest_syn.synthesize_ssml(ssml).await?;
//! ```
//!
//! Or you can synthesize text with [TextOptions][crate::TextOptions].
//!
//! ```ignore
//! use aspeak::TextOptionsBuilder;
//! let text = "Hello, world!";
//! let options = TextOptionsBuilder::new().voice("en-US-JennyNeural").rate("fast").pitch("high").build();
//! let audio_data = rest_syn.synthesize_text(text, &options).await?;
//! ```
//!
//! The full code can be found in [examples/03-rest-synthesizer-simple.rs](https://github.com/kxxt/aspeak/blob/main/examples/03-rest-synthesizer-simple.rs)
//!
//! ## Websocket Synthesizer
//!
//! You can also create a [WebsocketSynthesizer][crate::synthesizer::WebsocketSynthesizer].
//!
//! ```ignore
//! let mut ws_syn = config.connect_websocket().await?;
//! ```
//!
//! Then you can synthesize SSML to audio data.
//!
//! ```ignore
//! let ssml = r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US"><voice name="en-US-JennyNeural">Hello, world!</voice></speak>"#;
//! let audio_data = ws_syn.synthesize_ssml(ssml).await?;
//! ```
//!
//! or synthesize text with [TextOptions][crate::TextOptions].
//!
//! ```ignore
//! use aspeak::TextOptionsBuilder;
//! let text = "Hello, world!";
//! let options = TextOptionsBuilder::new().voice("en-US-JennyNeural").rate("fast").pitch("high").build();
//! let audio_data = ws_syn.synthesize_text(text, &options).await?;
//! ```
//!
//! The full code can be found in [examples/04-websocket-synthesizer-simple.rs](https://github.com/kxxt/aspeak/blob/main/examples/04-websocket-synthesizer-simple.rs)
//!
//! # Unified synthesizer trait
//!
//! There is also a unified synthesizer trait [Synthesizer][crate::synthesizer::UnifiedSynthesizer] that can be used to
//! provide a unified interface for both RESTful and Websocket synthesizers.
//!
//! # TLS feature flags
//!
//! By default, this crate uses `native-tls`. To use other TLS implementations, you can use the following feature flags:
//!
//! - `native-tls-vendored`: Use the vendored version of `native-tls`.
//! - `rustls-tls-native-roots`
//! - `rustls-tls-webpki-roots`
//!
//! Note that you need to disable the default features to disable `native-tls`. And after that, you need to manually enable your desired synthesizer features.
//!
//! # Feature flags
//!
//! - `rest-synthesizer`: Enable the RESTful synthesizer.
//! - `websocket-synthesizer`: Enable the Websocket synthesizer.
//! - `unified-synthesizer`: Enable the unified synthesizer trait.
//! - `synthesizers`: Enable all synthesizers.

mod audio;
mod auth;
mod constants;
mod errors;
#[cfg(feature = "websocket-synthesizer")]
mod msg;
#[cfg(feature = "websocket-synthesizer")]
mod net;
#[cfg(feature = "python")]
mod parse;
mod ssml;
pub mod synthesizer;
mod types;
mod utils;
pub mod voice;

/// Get the official websocket endpoint by its region (e.g. `eastus`)
pub fn get_websocket_endpoint_by_region(region: &str) -> String {
    format!("wss://{region}.tts.speech.microsoft.com/cognitiveservices/websocket/v1")
}

/// Get the official REST endpoint by its region (e.g. `eastus`)
pub fn get_rest_endpoint_by_region(region: &str) -> String {
    format!("https://{region}.tts.speech.microsoft.com/cognitiveservices/v1")
}

pub use audio::{AudioFormat, AudioFormatParseError, QUALITY_MAP, QUALITY_RANGE_MAP};
pub use auth::*;
use phf::phf_map;
pub use ssml::*;
pub use types::*;
pub use voice::Voice;

#[cfg(feature = "python")]
pub mod python;

/// Returns the default voice for the given locale.
///
/// # Argument
///
/// `locale`: A locale code like `en-US`.
/// Note that the country code is in uppercase and the language code is in lowercase.
///
/// # Returns
///
/// A `Result` that contains the default voice as a static string slice if the
/// specified locale is valid. Otherwise, an `AspeakError` is returned.
pub fn get_default_voice_by_locale(locale: &str) -> Option<&'static str> {
    DEFAULT_VOICES.get(locale).copied()
}

pub(crate) static DEFAULT_VOICES: phf::Map<&'static str, &'static str> = phf_map! {
    "af-ZA"=> "af-ZA-AdriNeural",
    "am-ET"=> "am-ET-AmehaNeural",
    "ar-AE"=> "ar-AE-FatimaNeural",
    "ar-BH"=> "ar-BH-AliNeural",
    "ar-DZ"=> "ar-DZ-AminaNeural",
    "ar-EG"=> "ar-EG-SalmaNeural",
    "ar-IQ"=> "ar-IQ-BasselNeural",
    "ar-JO"=> "ar-JO-SanaNeural",
    "ar-KW"=> "ar-KW-FahedNeural",
    "ar-LY"=> "ar-LY-ImanNeural",
    "ar-MA"=> "ar-MA-JamalNeural",
    "ar-QA"=> "ar-QA-AmalNeural",
    "ar-SA"=> "ar-SA-HamedNeural",
    "ar-SY"=> "ar-SY-AmanyNeural",
    "ar-TN"=> "ar-TN-HediNeural",
    "ar-YE"=> "ar-YE-MaryamNeural",
    "bg-BG"=> "bg-BG-BorislavNeural",
    "bn-BD"=> "bn-BD-NabanitaNeural",
    "bn-IN"=> "bn-IN-BashkarNeural",
    "ca-ES"=> "ca-ES-JoanaNeural",
    "cs-CZ"=> "cs-CZ-AntoninNeural",
    "cy-GB"=> "cy-GB-AledNeural",
    "da-DK"=> "da-DK-ChristelNeural",
    "de-AT"=> "de-AT-IngridNeural",
    "de-CH"=> "de-CH-JanNeural",
    "de-DE"=> "de-DE-KatjaNeural",
    "el-GR"=> "el-GR-AthinaNeural",
    "en-AU"=> "en-AU-NatashaNeural",
    "en-CA"=> "en-CA-ClaraNeural",
    "en-GB"=> "en-GB-LibbyNeural",
    "en-HK"=> "en-HK-SamNeural",
    "en-IE"=> "en-IE-ConnorNeural",
    "en-IN"=> "en-IN-NeerjaNeural",
    "en-KE"=> "en-KE-AsiliaNeural",
    "en-NG"=> "en-NG-AbeoNeural",
    "en-NZ"=> "en-NZ-MitchellNeural",
    "en-PH"=> "en-PH-JamesNeural",
    "en-SG"=> "en-SG-LunaNeural",
    "en-TZ"=> "en-TZ-ElimuNeural",
    "en-US"=> "en-US-JennyNeural",
    "en-ZA"=> "en-ZA-LeahNeural",
    "es-AR"=> "es-AR-ElenaNeural",
    "es-BO"=> "es-BO-MarceloNeural",
    "es-CL"=> "es-CL-CatalinaNeural",
    "es-CO"=> "es-CO-GonzaloNeural",
    "es-CR"=> "es-CR-JuanNeural",
    "es-CU"=> "es-CU-BelkysNeural",
    "es-DO"=> "es-DO-EmilioNeural",
    "es-EC"=> "es-EC-AndreaNeural",
    "es-ES"=> "es-ES-AlvaroNeural",
    "es-GQ"=> "es-GQ-JavierNeural",
    "es-GT"=> "es-GT-AndresNeural",
    "es-HN"=> "es-HN-CarlosNeural",
    "es-MX"=> "es-MX-DaliaNeural",
    "es-NI"=> "es-NI-FedericoNeural",
    "es-PA"=> "es-PA-MargaritaNeural",
    "es-PE"=> "es-PE-AlexNeural",
    "es-PR"=> "es-PR-KarinaNeural",
    "es-PY"=> "es-PY-MarioNeural",
    "es-SV"=> "es-SV-LorenaNeural",
    "es-US"=> "es-US-AlonsoNeural",
    "es-UY"=> "es-UY-MateoNeural",
    "es-VE"=> "es-VE-PaolaNeural",
    "et-EE"=> "et-EE-AnuNeural",
    "fa-IR"=> "fa-IR-DilaraNeural",
    "fi-FI"=> "fi-FI-SelmaNeural",
    "fil-PH"=> "fil-PH-AngeloNeural",
    "fr-BE"=> "fr-BE-CharlineNeural",
    "fr-CA"=> "fr-CA-SylvieNeural",
    "fr-CH"=> "fr-CH-ArianeNeural",
    "fr-FR"=> "fr-FR-DeniseNeural",
    "ga-IE"=> "ga-IE-ColmNeural",
    "gl-ES"=> "gl-ES-RoiNeural",
    "gu-IN"=> "gu-IN-DhwaniNeural",
    "he-IL"=> "he-IL-AvriNeural",
    "hi-IN"=> "hi-IN-MadhurNeural",
    "hr-HR"=> "hr-HR-GabrijelaNeural",
    "hu-HU"=> "hu-HU-NoemiNeural",
    "id-ID"=> "id-ID-ArdiNeural",
    "is-IS"=> "is-IS-GudrunNeural",
    "it-IT"=> "it-IT-IsabellaNeural",
    "ja-JP"=> "ja-JP-NanamiNeural",
    "jv-ID"=> "jv-ID-DimasNeural",
    "kk-KZ"=> "kk-KZ-AigulNeural",
    "km-KH"=> "km-KH-PisethNeural",
    "kn-IN"=> "kn-IN-GaganNeural",
    "ko-KR"=> "ko-KR-SunHiNeural",
    "lo-LA"=> "lo-LA-ChanthavongNeural",
    "lt-LT"=> "lt-LT-LeonasNeural",
    "lv-LV"=> "lv-LV-EveritaNeural",
    "mk-MK"=> "mk-MK-AleksandarNeural",
    "ml-IN"=> "ml-IN-MidhunNeural",
    "mr-IN"=> "mr-IN-AarohiNeural",
    "ms-MY"=> "ms-MY-OsmanNeural",
    "mt-MT"=> "mt-MT-GraceNeural",
    "my-MM"=> "my-MM-NilarNeural",
    "nb-NO"=> "nb-NO-PernilleNeural",
    "nl-BE"=> "nl-BE-ArnaudNeural",
    "nl-NL"=> "nl-NL-ColetteNeural",
    "pl-PL"=> "pl-PL-AgnieszkaNeural",
    "ps-AF"=> "ps-AF-GulNawazNeural",
    "pt-BR"=> "pt-BR-FranciscaNeural",
    "pt-PT"=> "pt-PT-DuarteNeural",
    "ro-RO"=> "ro-RO-AlinaNeural",
    "ru-RU"=> "ru-RU-SvetlanaNeural",
    "si-LK"=> "si-LK-SameeraNeural",
    "sk-SK"=> "sk-SK-LukasNeural",
    "sl-SI"=> "sl-SI-PetraNeural",
    "so-SO"=> "so-SO-MuuseNeural",
    "sr-RS"=> "sr-RS-NicholasNeural",
    "su-ID"=> "su-ID-JajangNeural",
    "sv-SE"=> "sv-SE-SofieNeural",
    "sw-KE"=> "sw-KE-RafikiNeural",
    "sw-TZ"=> "sw-TZ-DaudiNeural",
    "ta-IN"=> "ta-IN-PallaviNeural",
    "ta-LK"=> "ta-LK-KumarNeural",
    "ta-SG"=> "ta-SG-AnbuNeural",
    "te-IN"=> "te-IN-MohanNeural",
    "th-TH"=> "th-TH-PremwadeeNeural",
    "tr-TR"=> "tr-TR-AhmetNeural",
    "uk-UA"=> "uk-UA-OstapNeural",
    "ur-IN"=> "ur-IN-GulNeural",
    "ur-PK"=> "ur-PK-AsadNeural",
    "uz-UZ"=> "uz-UZ-MadinaNeural",
    "vi-VN"=> "vi-VN-HoaiMyNeural",
    "zh-CN"=> "zh-CN-XiaoxiaoNeural",
    "zh-HK"=> "zh-HK-HiuMaanNeural",
    "zh-TW"=> "zh-TW-HsiaoChenNeural",
    "zu-ZA"=> "zu-ZA-ThandoNeural",
};
