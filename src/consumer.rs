use crate::{RidoError, ValidateLanguage, ValidateWithArch, WindowsArchitecture, WindowsLanguage, WindowsRelease};
use reqwest::header::{ACCEPT, REFERER, USER_AGENT};
use serde::Deserialize;
use std::{fmt, time::SystemTime};
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
        _ => "",
    };
    let isotype = match arch {
        WindowsArchitecture::x86_64 => "x64",
        WindowsArchitecture::i686 => "x32",
    };

    let user_agent = {
        // Choose latest firefox release based on Firefox's 4 week release schedule
        let unix_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System clock is broken");
        let firefox_release = 124 + (unix_time.as_secs() - FIREFOX_124_RELEASE_TIME) / FOUR_WEEKS;
        format!("Mozilla 5.0 (X11, Linux x86_64; rv:{firefox_release}.0) Gecko/20100101 Firefox/{firefox_release}.0")
    };

    let uuid = Uuid::new_v4().to_string();

    let client = reqwest::blocking::Client::new();

    let product_id = if let ConsumerRelease::CustomProductID(id) = release {
        id.to_string()
    } else {
        let download_page_html = client
            .get(url)
            .header(USER_AGENT, &user_agent)
            .header(ACCEPT, "")
            .send()?
            .text()?;

        download_page_html[..std::cmp::min(download_page_html.len(), 102400)]
            .split("option")
            .find_map(|value| {
                let start = value.find("value=\"")? + 7;
                let end = value.find("\">Windows")?;
                Some(value.get(start..end).unwrap())
            })
            .ok_or(RidoError::ProductID)?
            .to_string()
    };

    client
        .get(format!("https://vlscppe.microsoft.com/tags?org_id=y6jn8c31&session_id={uuid}",))
        .header(ACCEPT, "")
        .header(USER_AGENT, &user_agent)
        .send()?;

    let skuid_table = get_skus(&client, &product_id, &uuid)?;
    let sku = skuid_table
        .into_iter()
        .find(|s| s.localized_language == lang.to_string())
        .ok_or(RidoError::SKUID)?;
    let skuid = sku.id;

    let referer = if let ConsumerRelease::CustomProductID(_) = release {
        // Product names are formatted as 'Windows <release> <release tag>'. e.g. 'Windows 11 24H2'
        // We can just grab the second space-delimited value and it should be release
        let release = sku.product_display_name.split_ascii_whitespace().nth(1).unwrap_or("11");
        &format!("https://www.microsoft.com/software-download/windows{release}")
    } else {
        url
    };
    let urls = get_urls(&client, &skuid, &uuid, referer)?;
    let url = urls
        .into_iter()
        .map(|u| u.uri)
        .find(|u| u.contains(isotype))
        .ok_or(RidoError::URL)?;
    Ok((url, None))
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SkuData {
    skus: Vec<WindowsSku>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WindowsSku {
    product_display_name: String,
    id: String,
    localized_language: String,
}

fn get_skus(client: &reqwest::blocking::Client, product_id: &str, uuid: &str) -> Result<Vec<WindowsSku>, RidoError> {
    let skuid_table_url = format!("https://www.microsoft.com/software-download-connector/api/getskuinformationbyproductedition?profile=606624d44113&ProductEditionId={}&SKU=undefined&friendlyFileName=undefined&Locale=en-US&sessionID={}", product_id,uuid);
    let skuid_table = client.get(skuid_table_url).send()?.text()?;
    let skuid_table: SkuData = serde_json::from_str(&skuid_table).map_err(RidoError::JSONParsing)?;
    Ok(skuid_table.skus)
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UrlDataParse {
    product_download_options: Vec<WindowsUrlData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WindowsUrlData {
    uri: String,
}

fn get_urls(client: &reqwest::blocking::Client, skuid: &str, uuid: &str, referer: &str) -> Result<Vec<WindowsUrlData>, RidoError> {
    let url = format!("https://www.microsoft.com/software-download-connector/api/GetProductDownloadLinksBySku?profile=606624d44113&productEditionId=undefined&SKU={skuid}&friendlyFileName=undefined&Locale=en-US&sessionID={uuid}");
    let url_json = client.get(url).header(REFERER, referer).send()?.text()?;
    let url: UrlDataParse = serde_json::from_str(&url_json).map_err(RidoError::JSONParsing)?;
    Ok(url.product_download_options)
}

#[derive(EnumIter, Debug, Copy, Clone, PartialEq)]
pub enum ConsumerRelease {
    Eleven,
    Ten,
    CustomProductID(u32),
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
            ConsumerRelease::CustomProductID(id) => &format!("Custom Product ID: {id}"),
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
            _ => {
                if let Some(Ok(product_id)) = value.strip_prefix("productid:").map(str::trim).map(str::parse) {
                    Self::CustomProductID(product_id)
                } else {
                    return Err(RidoError::InvalidReleaseStr);
                }
            }
        })
    }
}

#[derive(PartialEq, EnumIter, Debug, Copy, Clone)]
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
