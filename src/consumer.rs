use crate::{RidoError, ValidateLanguage, ValidateWithArch, WindowsArchitecture, WindowsLanguage, WindowsRelease};
use regex::Regex;
use reqwest::header::{ACCEPT, REFERER, USER_AGENT};
use std::{cmp::min, fmt, time::SystemTime};
use strum_macros::EnumIter;
use uuid::Uuid;

const FIREFOX_124_RELEASE_TIME: u64 = 1710806400;
const FOUR_WEEKS: u64 = 2419200;

pub fn get_consumer_info(release: ConsumerRelease, lang: ConsumerLanguage, arch: WindowsArchitecture) -> Result<(String, Option<String>), RidoError> {
    if arch == WindowsArchitecture::i686 && release == ConsumerRelease::Eleven {
        return Err(RidoError::InvalidArchitecture(release.into(), arch));
    }
    let url = match release {
        ConsumerRelease::Ten => "https://microsoft.com/en-us/software-download/windows10ISO",
        ConsumerRelease::Eleven => "https://microsoft.com/en-us/software-download/windows11",
    };
    let (isotype, bits) = match arch {
        WindowsArchitecture::x86_64 => ("x64", "64"),
        WindowsArchitecture::i686 => ("x32", "32"),
    };

    let lang_binding = lang.to_string();
    let hash_lang = match lang {
        ConsumerLanguage::EnglishUS => "English",
        ConsumerLanguage::SimplifiedChinese => "Chinese Simplified",
        ConsumerLanguage::TraditionalChinese => "Chinese Traditional",
        _ => &lang_binding,
    };

    let user_agent = {
        // Choose latest firefox release based on Firefox's 4 week release schedule
        let unix_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System clock is broken");
        let firefox_release = 124 + (unix_time.as_secs() - FIREFOX_124_RELEASE_TIME) / FOUR_WEEKS;
        format!("Mozilla 5.0 (X11, Linux x86_64; rv:{firefox_release}.0) Gecko/20100101 Firefox/{firefox_release}.0")
    };

    let uuid = Uuid::new_v4();

    let client = reqwest::blocking::Client::new();

    let download_page_html = client
        .get(url)
        .header(USER_AGENT, &user_agent)
        .header(ACCEPT, "")
        .send()?
        .text()?;

    let product_id = download_page_html[..std::cmp::min(download_page_html.len(), 102400)]
        .split("option")
        .find_map(|value| {
            let start = value.find("value=\"")? + 7;
            let end = value.find("\">Windows")?;
            Some(value.get(start..end).unwrap())
        })
        .ok_or(RidoError::ProductID)?;

    client
        .get(format!("https://vlscppe.microsoft.com/tags?org_id=y6jn8c31&session_id={uuid}",))
        .header(ACCEPT, "")
        .header(USER_AGENT, &user_agent)
        .send()?;

    let url_segment = &url.split('/').last().unwrap();
    let skuid_table = client.post(format!("https://www.microsoft.com/en-US/api/controls/contentinclude/html?pageId=a8f8f489-4c7f-463a-9ca6-5cff94d8d041&host=www.microsoft.com&segments=software-download,{}&query=&action=getskuinformationbyproductedition&sessionId={}&productEditionId={}&sdVersion=2", url_segment, uuid, product_id))
        .header(USER_AGENT, &user_agent)
        .header(ACCEPT, "")
        .header(REFERER, url)
        .body("")
        .send()?
        .text()?;

    let skuid = skuid_table[..std::cmp::min(skuid_table.len(), 10240)]
        .lines()
        .find(|line| line.contains(&lang.to_string()))
        .ok_or(RidoError::SKUID)?
        .split("&quot;")
        .nth(3)
        .unwrap();

    let download_page_url = match release {
        ConsumerRelease::Ten => format!("https://www.microsoft.com/en-us/api/controls/contentinclude/html?pageId=a224afab-2097-4dfa-a2ba-463eb191a285&host=www.microsoft.com&segments=software-download,windows10ISO&query=&action=GetProductDownloadLinksBySku&sessionId={uuid}&skuId={skuid}&language=English&sdVersion=2"),
        ConsumerRelease::Eleven => format!("https://www.microsoft.com/en-US/api/controls/contentinclude/html?pageId=6e2a1789-ef16-4f27-a296-74ef7ef5d96b&host=www.microsoft.com&segments=software-download,windows11&query=&action=GetProductDownloadLinksBySku&sessionId={uuid}&skuId={skuid}&language=English&sdVersion=2"),
    };

    let download_link_html = client
        .post(download_page_url)
        .header(USER_AGENT, &user_agent)
        .header(ACCEPT, "")
        .header(REFERER, url)
        .body("")
        .send()?
        .text()?;

    if download_link_html.is_empty() {
        return Err(RidoError::EmptyResponse);
    } else if download_link_html.contains("We are unable to complete your request at this time.") {
        return Err(RidoError::BlockedRequest);
    }

    let hash_regex = Regex::new(r#">([A-F0-9]{64})</td></tr><tr><td>([^36]*)(64|32)-bit"#).unwrap();
    let hash = hash_regex.captures_iter(&download_link_html).find_map(
        |c| {
            if c[2].trim() == hash_lang && &c[3] == bits {
                Some(c[1].to_string())
            } else {
                None
            }
        },
    );

    let html_truncation_len = min(download_link_html.len(), 4096);

    let url_regex = Regex::new(r#"href="(https://software\.download\.prss\.microsoft\.com/dbazure[^"]*)""#).unwrap();
    let url_capture = url_regex
        .captures_iter(&download_link_html[..html_truncation_len])
        .find(|c| {
            let url = &c[1];
            url.find(".iso").map_or(false, |index| url[..index].contains(isotype))
        })
        .ok_or(RidoError::HTMLParse)?;

    let url = url_capture[1]
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
        .collect::<String>()
        .replace("&amp;", "&");

    Ok((url, hash))
}

#[derive(EnumIter, Debug, Copy, Clone, PartialEq)]
pub enum ConsumerRelease {
    Ten,
    Eleven,
}

impl From<ConsumerRelease> for WindowsRelease {
    fn from(release: ConsumerRelease) -> Self {
        Self::Consumer(release)
    }
}

impl fmt::Display for ConsumerRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ConsumerRelease::Ten => "Windows 10",
            ConsumerRelease::Eleven => "Windows 11",
        };
        write!(f, "{text}")
    }
}

impl TryFrom<&str> for ConsumerRelease {
    type Error = RidoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "10" => Self::Ten,
            "11" => Self::Eleven,
            _ => return Err(RidoError::InvalidReleaseStr),
        })
    }
}

#[derive(EnumIter, Debug, Copy, Clone)]
pub enum ConsumerLanguage {
    Arabic,
    BrazilianPortuguese,
    Bulgarian,
    Croatian,
    Czech,
    Danish,
    Dutch,
    EnglishInternational,
    EnglishUS,
    Estonian,
    Finnish,
    French,
    FrenchCanadian,
    German,
    Greek,
    Hebrew,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Latvian,
    Lithuanian,
    MexicanSpanish,
    Norwegian,
    Polish,
    Portuguese,
    Romanian,
    Russian,
    SerbianLatin,
    SimplifiedChinese,
    Slovak,
    Slovenian,
    Spanish,
    Swedish,
    Thai,
    TraditionalChinese,
    Turkish,
    Ukrainian,
}

impl From<ConsumerLanguage> for WindowsLanguage {
    fn from(lang: ConsumerLanguage) -> Self {
        Self::Consumer(lang)
    }
}

impl fmt::Display for ConsumerLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Arabic => "Arabic",
            Self::BrazilianPortuguese => "Brazilian Portuguese",
            Self::Bulgarian => "Bulgarian",
            Self::Croatian => "Croatian",
            Self::Czech => "Czech",
            Self::Danish => "Danish",
            Self::Dutch => "Dutch",
            Self::EnglishInternational => "English International",
            Self::EnglishUS => "English (United States)",
            Self::Estonian => "Estonian",
            Self::Finnish => "Finnish",
            Self::French => "French",
            Self::FrenchCanadian => "French Canadian",
            Self::German => "German",
            Self::Greek => "Greek",
            Self::Hebrew => "Hebrew",
            Self::Hungarian => "Hungarian",
            Self::Italian => "Italian",
            Self::Japanese => "Japanese",
            Self::Korean => "Korean",
            Self::Latvian => "Latvian",
            Self::Lithuanian => "Lithuanian",
            Self::MexicanSpanish => "Spanish (Mexico)",
            Self::Norwegian => "Norwegian",
            Self::Polish => "Polish",
            Self::Portuguese => "Portuguese",
            Self::Romanian => "Romanian",
            Self::Russian => "Russian",
            Self::SerbianLatin => "Serbian Latin",
            Self::SimplifiedChinese => "Chinese (Simplified)",
            Self::Slovak => "Slovak",
            Self::Slovenian => "Slovenian",
            Self::Spanish => "Spanish",
            Self::Swedish => "Swedish",
            Self::Thai => "Thai",
            Self::TraditionalChinese => "Chinese (Traditional)",
            Self::Turkish => "Turkish",
            Self::Ukrainian => "Ukrainian",
        };
        write!(f, "{text}")
    }
}

impl TryFrom<&str> for ConsumerLanguage {
    type Error = RidoError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Arabic" => Self::Arabic,
            "Brazilian Portuguese" => Self::BrazilianPortuguese,
            "Bulgarian" => Self::Bulgarian,
            "Croatian" => Self::Croatian,
            "Czech" => Self::Czech,
            "Danish" => Self::Danish,
            "Dutch" => Self::Dutch,
            "English International" => Self::EnglishInternational,
            "English (United States)" => Self::EnglishUS,
            "Estonian" => Self::Estonian,
            "Finnish" => Self::Finnish,
            "French" => Self::French,
            "French Canadian" => Self::FrenchCanadian,
            "German" => Self::German,
            "Greek" => Self::Greek,
            "Hebrew" => Self::Hebrew,
            "Hungarian" => Self::Hungarian,
            "Italian" => Self::Italian,
            "Japanese" => Self::Japanese,
            "Korean" => Self::Korean,
            "Latvian" => Self::Latvian,
            "Lithuanian" => Self::Lithuanian,
            "Spanish (Mexico)" => Self::MexicanSpanish,
            "Norwegian" => Self::Norwegian,
            "Polish" => Self::Polish,
            "Portuguese" => Self::Portuguese,
            "Romanian" => Self::Romanian,
            "Russian" => Self::Russian,
            "Serbian Latin" => Self::SerbianLatin,
            "Chinese (Simplified)" => Self::SimplifiedChinese,
            "Slovak" => Self::Slovak,
            "Slovenian" => Self::Slovenian,
            "Spanish" => Self::Spanish,
            "Swedish" => Self::Swedish,
            "Thai" => Self::Thai,
            "Chinese (Traditional)" => Self::TraditionalChinese,
            "Turkish" => Self::Turkish,
            "Ukrainian" => Self::Ukrainian,
            _ => return Err(RidoError::InvalidLanguageStr),
        })
    }
}

impl ValidateLanguage for ConsumerLanguage {
    fn validate(&self, release: WindowsRelease) -> bool {
        matches!(release, WindowsRelease::Consumer(_))
    }
}

impl ValidateWithArch for ConsumerRelease {
    fn validate(&self, arch: WindowsArchitecture) -> bool {
        !matches!(self, ConsumerRelease::Eleven if arch == WindowsArchitecture::i686)
    }
}
