use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use thiserror::Error;

#[cfg(feature = "consumer")]
mod consumer;
#[cfg(feature = "consumer")]
pub use consumer::{ConsumerLanguage, ConsumerRelease};

#[cfg(feature = "enterprise")]
mod enterprise;
#[cfg(feature = "enterprise")]
pub use enterprise::{EnterpriseLanguage, EnterpriseRelease};

#[derive(Debug, Clone)]
pub struct WindowsData {
    pub info: WindowsEntry,
    pub url: String,
    pub hash: Option<String>,
}

impl WindowsData {
    pub fn new<R, L, A>(release: R, lang: L, arch: A) -> Result<Self, RidoError>
    where
        WindowsLanguage: TryFrom<(WindowsRelease, L), Error = RidoError>,
        R: TryInto<WindowsRelease, Error = RidoError>,
        A: TryInto<WindowsArchitecture, Error = RidoError>,
    {
        let release = release.try_into()?;
        let lang = (release, lang).try_into()?;
        let arch = arch.try_into()?;
        WindowsEntry { release, lang, arch }.try_into()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct WindowsEntry {
    pub release: WindowsRelease,
    pub arch: WindowsArchitecture,
    pub lang: WindowsLanguage,
}

impl WindowsEntry {
    pub fn list_all() -> Vec<Self> {
        let mut entries = Vec::new();

        #[cfg(feature = "consumer")]
        entries.extend(ConsumerRelease::iter().flat_map(move |rel| {
            let release: WindowsRelease = rel.into();
            ConsumerLanguage::iter()
                .filter(move |&lang| lang.validate(release))
                .flat_map(move |lang| {
                    let lang = lang.into();
                    WindowsArchitecture::iter()
                        .filter(move |&arch| rel.validate(arch))
                        .map(move |arch| Self { release, lang, arch })
                })
        }));

        #[cfg(feature = "enterprise")]
        entries.extend(EnterpriseRelease::iter().flat_map(move |rel| {
            let release: WindowsRelease = rel.into();
            EnterpriseLanguage::iter()
                .filter(move |&lang| lang.validate(release))
                .flat_map(move |lang| {
                    let lang = lang.into();
                    WindowsArchitecture::iter()
                        .filter(move |&arch| rel.validate(arch))
                        .map(move |arch| Self { release, lang, arch })
                })
        }));

        entries
    }
}

impl TryFrom<WindowsEntry> for WindowsData {
    type Error = RidoError;
    fn try_from(entry: WindowsEntry) -> Result<Self, Self::Error> {
        let (url, hash) = match (entry.release, entry.lang) {
            #[cfg(feature = "consumer")]
            (WindowsRelease::Consumer(release), WindowsLanguage::Consumer(lang)) => consumer::get_consumer_info(release, lang, entry.arch)?,
            #[cfg(feature = "enterprise")]
            (WindowsRelease::Enterprise(release), WindowsLanguage::Enterprise(lang)) => enterprise::get_enterprise_info(release, lang, entry.arch)?,
            #[allow(unreachable_patterns)]
            _ => return Err(RidoError::InvalidSelection),
        };
        Ok(Self { info: entry, url, hash })
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum WindowsRelease {
    #[cfg(feature = "consumer")]
    Consumer(ConsumerRelease),
    #[cfg(feature = "enterprise")]
    Enterprise(EnterpriseRelease),
}
impl fmt::Display for WindowsRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "consumer")]
            Self::Consumer(release) => write!(f, "{release}"),
            #[cfg(feature = "enterprise")]
            Self::Enterprise(release) => write!(f, "{release}"),
        }
    }
}
impl TryFrom<&str> for WindowsRelease {
    type Error = RidoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        #[cfg(feature = "consumer")]
        if let Ok(release) = value.try_into() {
            return Ok(Self::Consumer(release));
        }
        #[cfg(feature = "enterprise")]
        if let Ok(release) = value.try_into() {
            return Ok(Self::Enterprise(release));
        }
        Err(RidoError::InvalidReleaseStr)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum WindowsLanguage {
    #[cfg(feature = "consumer")]
    Consumer(ConsumerLanguage),
    #[cfg(feature = "enterprise")]
    Enterprise(EnterpriseLanguage),
}
impl fmt::Display for WindowsLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "consumer")]
            Self::Consumer(lang) => write!(f, "{lang}"),
            #[cfg(feature = "enterprise")]
            Self::Enterprise(lang) => write!(f, "{lang}"),
        }
    }
}
impl TryFrom<(WindowsRelease, &str)> for WindowsLanguage {
    type Error = RidoError;
    fn try_from(value: (WindowsRelease, &str)) -> Result<Self, Self::Error> {
        match value.0 {
            #[cfg(feature = "consumer")]
            WindowsRelease::Consumer(_) => ConsumerLanguage::try_from(value.1).map(Into::into),
            #[cfg(feature = "enterprise")]
            WindowsRelease::Enterprise(_) => EnterpriseLanguage::try_from(value.1).map(Into::into),
        }
    }
}
impl<L> TryFrom<(WindowsRelease, L)> for WindowsLanguage
where
    L: Into<WindowsLanguage>,
{
    type Error = RidoError;
    fn try_from(value: (WindowsRelease, L)) -> Result<Self, Self::Error> {
        Ok(value.1.into())
    }
}

#[allow(non_camel_case_types)]
#[derive(EnumIter, Debug, Display, Copy, Clone, PartialEq)]
pub enum WindowsArchitecture {
    x86_64,
    i686,
}
impl TryFrom<&str> for WindowsArchitecture {
    type Error = RidoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "i386" | "i686" | "x86" | "x32" => Self::i686,
            "x86_64" | "amd64" | "x64" => Self::x86_64,
            _ => return Err(RidoError::InvalidArchitectureStr),
        })
    }
}

#[derive(Debug, Error)]
pub enum RidoError {
    #[error("Specified architecture {1} is not available for release {0}")]
    InvalidArchitecture(WindowsRelease, WindowsArchitecture),
    #[error("Specified language {1} is not available for release {0}")]
    InvalidLanguage(WindowsRelease, WindowsLanguage),
    #[error("Invalid release")]
    InvalidReleaseStr,
    #[error("Invalid language")]
    InvalidLanguageStr,
    #[error("Invalid architecture")]
    InvalidArchitectureStr,
    #[error("The language type must match the release type (Enterprise/Consumer)")]
    InvalidSelection,
    #[error("Microsoft servers gave us an empty response to our request for an automated download.")]
    EmptyResponse,
    #[error("Microsoft blocked the automated download request based on your IP address.")]
    BlockedRequest,
    #[error("Unable to parse download link from HTML")]
    HTMLParse,
    #[error("Could not parse JSON: {0}")]
    JSONParsing(serde_json::Error),
    #[error("Could not find SKUID")]
    SKUID,
    #[error("Could not find Product ID")]
    ProductID,
    #[error("Could not find URL")]
    URL,
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

pub(crate) trait ValidateLanguage {
    fn validate(&self, release: WindowsRelease) -> bool;
}
pub(crate) trait ValidateWithArch {
    fn validate(&self, arch: WindowsArchitecture) -> bool;
}
