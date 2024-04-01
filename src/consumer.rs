use uuid::Uuid;
use reqwest::header::{ACCEPT, USER_AGENT, REFERER};
use std::time::SystemTime;
use std::error::Error;

pub fn get_consumer_info(release: &str, language: &str, arch: &str) -> Result<(String, Option<String>), Box<dyn Error>> {
    if arch == "i686" && release == "11" {
        return Err("Windows 11 does not offer a 32-bit edition.".into());
    }
    let url = match release {
        "8" => "https://microsoft.com/en-us/software-download/windows8ISO",
        "10" => "https://microsoft.com/en-us/software-download/windows10ISO",
        "11" => "https://microsoft.com/en-us/software-download/windows11",
        _ => return Err("Unsupported release".into()),
    };
    let (isotype, bits) = match arch {
        // 'English (United States)' checksums appear as 'English' on the download page.
        "x86_64" => ("IsoX64", "64-bit"),
        "i686" => ("IsoX86", "32-bit"),
        _ => return Err("Unsupported architecture.".into()),
    };

    let hash_lang = match language {
        "English (United States)" => "English",
        "Chinese (Simplified)" => "Chinese Simplified",
        "Chinese (Traditional)" => "Chinese Traditional",
        _ => language,
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

    let download_page_url = match release {
        "8" => format!("https://www.microsoft.com/en-us/api/controls/contentinclude/html?pageId=cfa9e580-a81e-4a4b-a846-7b21bf4e2e5b&host=www.microsoft.com&segments=software-download%2cwindows8ISO&query=&action=GetProductDownloadLinksBySku&sessionId={}&skuId={}&language=English&sdVersion=2", uuid, skuid),
        "10" => format!("https://www.microsoft.com/en-us/api/controls/contentinclude/html?pageId=a224afab-2097-4dfa-a2ba-463eb191a285&host=www.microsoft.com&segments=software-download,windows10ISO&query=&action=GetProductDownloadLinksBySku&sessionId={}&skuId={}&language=English&sdVersion=2", uuid, skuid),
        "11" => format!("https://www.microsoft.com/en-US/api/controls/contentinclude/html?pageId=6e2a1789-ef16-4f27-a296-74ef7ef5d96b&host=www.microsoft.com&segments=software-download,windows11&query=&action=GetProductDownloadLinksBySku&sessionId={}&skuId={}&language=English&sdVersion=2", uuid, skuid),
        _ => return Err("Unsupported release".into()),
    };

    let download_link_html = client.post(download_page_url)
        .header(USER_AGENT, &useragent)
        .header(ACCEPT, "")
        .header(REFERER, url)
        .body("")
        .send().map_err(|e| format!("{} while trying to find the download link.", e))?
        .text()?;

    let hash = download_link_html.split("<tr><td>").find_map(|line| {
        println!("{}", line);
        if line.contains(&hash_lang) && line.contains(bits) {
            match release {
                "11" => Some(line.split(r#""word-wrap: break-word">"#).nth(1).unwrap()
                    .split("</td>").next().unwrap().to_string()),
                "10" => Some(line.split("</td><td>").nth(1).unwrap()
                    .split("</td></tr>").next().unwrap().to_string()),
                _ => None,
            }
        } else {
            None
        }
    });

    let download_link_html = &download_link_html[..std::cmp::min(download_link_html.len(), 4096)];

    if download_link_html.is_empty() {
        return Err("Microsoft servers gave us an empty response to our request for an automated odwnload.".into());
    } else if download_link_html.contains("We are unable to complete your request at this time.") {
        return Err("Microsoft blocked the automated download request based on your IP address.".into());
    }

    let ending = download_link_html.find(isotype).ok_or("Unable to parse download link.")?;
    let starting = download_link_html[..ending].rfind("https://software.download.prss.microsoft.com").ok_or("Unable to parse download link.")?;

    let mut link = download_link_html[starting..ending].chars()
        .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
        .collect::<String>()
        .replace("&amp;", "&");
    link.truncate(512);

    Ok((link, hash))
}
