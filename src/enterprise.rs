use crate::{RidoError, ValidateLanguage, ValidateWithArch, WindowsArchitecture, WindowsLanguage, WindowsRelease};
use regex::Regex;
use std::fmt;
use strum_macros::EnumIter;

pub fn get_enterprise_info(release: EnterpriseRelease, lang: EnterpriseLanguage, arch: WindowsArchitecture) -> Result<(String, Option<String>), RidoError> {
    if !lang.validate(release.into()) {
        return Err(RidoError::InvalidLanguage(release.into(), lang.into()));
    }
    let valid_release = match release {
        EnterpriseRelease::TenEnterprise | EnterpriseRelease::TenLtsc => "windows-10-enterprise",
        EnterpriseRelease::ElevenEnterprise => "windows-11-enterprise",
        EnterpriseRelease::Server2012R2 => "windows-server-2012-r2",
        EnterpriseRelease::Server2016 => "windows-server-2016",
        EnterpriseRelease::Server2019 => "windows-server-2019",
        EnterpriseRelease::Server2022 => "windows-server-2022",
    };

    let url = format!("https://www.microsoft.com/en-us/evalcenter/download-{valid_release}");

    let client = reqwest::blocking::Client::new();
    let download_page_html = client.get(url).send()?.text()?;

    if download_page_html.is_empty() {
        return Err(RidoError::EmptyResponse);
    }

    let (culture, country) = match lang {
        EnterpriseLanguage::BrazilianPortuguese => ("pt-br", "BR"),
        EnterpriseLanguage::EnglishGB => ("en-gb", "GB"),
        EnterpriseLanguage::EnglishUS => ("en-us", "US"),
        EnterpriseLanguage::French => ("fr-fr", "FR"),
        EnterpriseLanguage::German => ("de-de", "DE"),
        EnterpriseLanguage::Italian => ("it-it", "IT"),
        EnterpriseLanguage::Japanese => ("ja-jp", "JP"),
        EnterpriseLanguage::Korean => ("ko-kr", "KR"),
        EnterpriseLanguage::Russian => ("ru-ru", "RU"),
        EnterpriseLanguage::SimplifiedChinese => ("zh-cn", "CN"),
        EnterpriseLanguage::Spanish => ("es-es", "ES"),
        EnterpriseLanguage::TraditionalChinese => ("zh-tw", "TW"),
    };

    let bits = match arch {
        WindowsArchitecture::i686 => "32",
        WindowsArchitecture::x86_64 => "64",
    };

    let iso_regex = Regex::new(r#"href="(https://go\.microsoft\.com/fwlink/p/\?LinkID=\d{7}&clcid=0x\w{3}&culture=([a-z]{2}-[a-z]{2})&country=(\w{2}))">\s(64|32)-bit"#).unwrap();

    let urls = iso_regex
        .captures_iter(&download_page_html)
        .filter(|c| c[2] == *culture && c[3] == *country && c[4] == *bits)
        .collect::<Vec<_>>();

    let url_capture = match release {
        EnterpriseRelease::TenLtsc => urls.get(1),
        _ => urls.first(),
    }
    .ok_or(RidoError::HTMLParse)?;

    let url = url_capture[1].to_string();

    Ok((url, None))
}

#[derive(PartialEq, EnumIter, Debug, Copy, Clone)]
pub enum EnterpriseRelease {
    ElevenEnterprise,
    TenEnterprise,
    TenLtsc,
    Server2022,
    Server2019,
    Server2016,
    Server2012R2,
}

impl From<EnterpriseRelease> for WindowsRelease {
    fn from(release: EnterpriseRelease) -> Self {
        Self::Enterprise(release)
    }
}

impl fmt::Display for EnterpriseRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            EnterpriseRelease::TenEnterprise => "Windows 10 Enterprise",
            EnterpriseRelease::TenLtsc => "Windows 10 LTSC",
            EnterpriseRelease::ElevenEnterprise => "Windows 11 Enterprise",
            EnterpriseRelease::Server2012R2 => "Windows Server 2012 R2",
            EnterpriseRelease::Server2016 => "Windows Server 2016",
            EnterpriseRelease::Server2019 => "Windows Server 2019",
            EnterpriseRelease::Server2022 => "Windows Server 2022",
        };
        write!(f, "{text}")
    }
}

impl TryFrom<&str> for EnterpriseRelease {
    type Error = RidoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "10-enterprise" => Self::TenEnterprise,
            "10-ltsc" => Self::TenLtsc,
            "11-enterprise" => Self::ElevenEnterprise,
            "server-2012-r2" => Self::Server2012R2,
            "server-2016" => Self::Server2016,
            "server-2019" => Self::Server2019,
            "server-2022" => Self::Server2022,
            _ => return Err(RidoError::InvalidReleaseStr),
        })
    }
}

#[derive(PartialEq, EnumIter, Debug, Copy, Clone)]
pub enum EnterpriseLanguage {
    BrazilianPortuguese,
    EnglishUS,
    EnglishGB,
    French,
    German,
    Italian,
    Japanese,
    Korean,
    Russian,
    SimplifiedChinese,
    Spanish,
    TraditionalChinese,
}

impl From<EnterpriseLanguage> for WindowsLanguage {
    fn from(lang: EnterpriseLanguage) -> Self {
        Self::Enterprise(lang)
    }
}

impl fmt::Display for EnterpriseLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::BrazilianPortuguese => "Portuguese (Brazil)",
            Self::EnglishUS => "English (United States)",
            Self::EnglishGB => "English (Great Britain)",
            Self::French => "French",
            Self::German => "German",
            Self::Italian => "Italian",
            Self::Japanese => "Japanese",
            Self::Korean => "Korean",
            Self::Russian => "Russian",
            Self::SimplifiedChinese => "Chinese (Simplified)",
            Self::Spanish => "Spanish",
            Self::TraditionalChinese => "Chinese (Traditional)",
        };
        write!(f, "{text}")
    }
}

impl TryFrom<&str> for EnterpriseLanguage {
    type Error = RidoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Portuguese (Brazil)" => Self::BrazilianPortuguese,
            "English (United States)" => Self::EnglishUS,
            "English (Great Britain)" => Self::EnglishGB,
            "French" => Self::French,
            "German" => Self::German,
            "Italian" => Self::Italian,
            "Japanese" => Self::Japanese,
            "Korean" => Self::Korean,
            "Russian" => Self::Russian,
            "Chinese (Simplified)" => Self::SimplifiedChinese,
            "Spanish" => Self::Spanish,
            "Chinese (Traditional)" => Self::TraditionalChinese,
            _ => return Err(RidoError::InvalidLanguageStr),
        })
    }
}

impl ValidateLanguage for EnterpriseLanguage {
    fn validate(&self, release: WindowsRelease) -> bool {
        match release {
            WindowsRelease::Enterprise(release) => !match release {
                EnterpriseRelease::TenLtsc | EnterpriseRelease::TenEnterprise => matches!(self, EnterpriseLanguage::Russian),
                _ => matches!(
                    self,
                    EnterpriseLanguage::BrazilianPortuguese | EnterpriseLanguage::EnglishGB | EnterpriseLanguage::Korean | EnterpriseLanguage::TraditionalChinese
                ),
            },
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

impl ValidateWithArch for EnterpriseRelease {
    fn validate(&self, arch: WindowsArchitecture) -> bool {
        if let EnterpriseRelease::TenLtsc | EnterpriseRelease::TenEnterprise = self {
            true
        } else {
            arch == WindowsArchitecture::x86_64
        }
    }
}
