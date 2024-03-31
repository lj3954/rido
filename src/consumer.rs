use uuid::Uuid;
use reqwest::header::{ACCEPT, USER_AGENT, REFERER};
use std::time::SystemTime;
use std::error::Error;

pub struct ConsumerRelease {
    pub url: String,
    pub hash: String,
}

impl ConsumerRelease {
    pub fn new(release: &str, lang: &str) -> Result<Self, Box<dyn Error>> {
        let (url, hash) = get_consumer_info(release, lang)?;
        Ok(Self {
            url,
            hash,
        })
    }
}

pub fn get_consumer_info(release: &str, language: &str) -> Result<(String, String), Box<dyn Error>> {
    let url = match release {
        "8" => "https://microsoft.com/en-us/software-download/windows8ISO",
        "10" => "https://microsoft.com/en-us/software-download/windows10ISO",
        "11" => "https://microsoft.com/en-us/software-download/windows11",
        _ => return Err("Unsupported release".into()),
    };

    // Choose latest firefox release based on Firefox's 4 week release schedule
    let firefox_release = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        // Subtract March 19, 2024, release date of Firefox 124. Then, divide by 4 weeks.
        Ok(time) => 124 + (time.as_secs() - 1710806400) / 2419200,
        Err(_) => return Err("Invalid system time.".into()),
    };
    let useragent = format!("Mozilla 5.0 (X11, Linux x86_64; rv:{}.0) Gecko/20100101 Firefox/{}.0", firefox_release, firefox_release);
    let uuid = Uuid::new_v4();

    let client = reqwest::blocking::Client::new();
    
    let download_page_html = client.get(url)
        .header(USER_AGENT, &useragent)
        .header(ACCEPT, "")
        .send()?
        .text()?;
    
    let product_id = download_page_html[..std::cmp::min(download_page_html.len(), 102400)].split("option").find_map(|value| {
        let start = value.find("value=\"")? + 7;
        let end = value.find("\">Windows")?;
        Some(value.get(start..end).unwrap())
    }).ok_or("Could not find product ID.")?;

    client.get(format!("https://vlscppe.microsoft.com/tags?org_id=y6jn8c31&session_id={}", uuid))
        .header(ACCEPT, "")
        .header(USER_AGENT, &useragent)
        .send()?;

    let url_segment = &url.split("/").last().unwrap();
    let skuid_table = client.post(format!("https://www.microsoft.com/en-US/api/controls/contentinclude/html?pageId=a8f8f489-4c7f-463a-9ca6-5cff94d8d041&host=www.microsoft.com&segments=software-download,{}&query=&action=getskuinformationbyproductedition&sessionId={}&productEditionId={}&sdVersion=2", url_segment, uuid, product_id))
        .header(USER_AGENT, &useragent)
        .header(ACCEPT, "")
        .header(REFERER, url)
        .body("")
        .send()?
        .text()?;

    let skuid = skuid_table[..std::cmp::min(skuid_table.len(), 10240)].lines().find(|line| line.contains(language))
        .ok_or("Could not find skuid.")?
        .split("&quot;").nth(3).unwrap();

    let download_link_html = client.post(format!("https://www.microsoft.com/en-US/api/controls/contentinclude/html?pageId=6e2a1789-ef16-4f27-a296-74ef7ef5d96b&host=www.microsoft.com&segments=software-download,{}&query=&action=GetProductDownloadLinksBySku&sessionId={}&skuId={}&language=English&sdVersion=2", url_segment, uuid, skuid))
        .header(USER_AGENT, &useragent)
        .header(ACCEPT, "")
        .header(REFERER, url)
        .body("")
        .send().map_err(|e| format!("{} while trying to find the download link.", e))?
        .text()?;
    let download_link_html = &download_link_html[..std::cmp::min(download_link_html.len(), 4096)];

    if download_link_html.is_empty() {
        return Err("Microsoft servers gave us an empty response to our request for an automated odwnload.".into());
    } else if download_link_html.contains("We are unable to complete your request at this time.") {
        return Err("Microsoft blocked the automated download request based on your IP address.".into());
    }

    let ending = download_link_html.find("IsoX64").ok_or("Unable to parse download link.")?;
    let Some(starting) = download_link_html[..ending].rfind("https://software.download.prss.microsoft.com")
        else {
            return Err("Unable to parse download link from HTML.".into());
        };

    let link = download_link_html[starting..ending].chars()
        .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
        .collect::<String>()
        .replace("&amp;", "&");

    let hash = match release {
        "11" => download_link_html.split("<tr><td>").find(|line| {
            line.contains(&(language.to_owned() + "  64-bit"))
        }).unwrap_or(""),
        _ => "",
    };

    Ok((link, hash.to_string()))
}


