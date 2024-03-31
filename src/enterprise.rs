use std::error::Error;

pub fn get_enterprise_info(release: &str, lang: &str, arch: &str)-> Result<(String, String), Box<dyn Error>> {
    let valid_release = match release {
        "10-ltsc" => "windows-10-enterprise".into(),
        _ => "windows-".to_owned() + release,
    };

    let url = format!("https://www.microsoft.com/en-us/evalcenter/download-{}", valid_release);

    let client = reqwest::blocking::Client::new();
    let download_page_html = client.get(&url).send()?.text()?;
    
    if download_page_html.is_empty() {
        return Err("Windows enterprise evaluation download page gave us an empty response".into());
    }

    let (culture, country) = match lang {
        "English (Great Britain)" => ("en-gb", "GB"),
        "Chinese (Simplified)" => ("zh-cn", "CN"),
        "Chinese (Traditional)" => ("zh-tw", "TW"),
        "French" => ("fr-fr", "FR"),
        "German" => ("de-de", "DE"),
        "Italian" => ("it-it", "IT"),
        "Japanese" => ("ja-jp", "JP"),
        "Korean" => ("ko-kr", "KR"),
        "Portuguese (Brazil)" => ("pt-br", "BR"),
        "Spanish" => ("es-es", "ES"),
        "Russian" => ("ru-ru", "RU"),
        _ => ("en-us", "US"),
    };

    let iso_download_links = download_page_html.split("class=\"cta font-weight-semibold \" data-target=\"").filter_map(|line| {
        if line.contains(&format!("&culture={}&country={}", culture, country)) {
            Some(line.split(country).next().unwrap())
        } else {
            None
        }
    }).collect::<Vec<&str>>();

    let link = match (release, arch) {
        ("10-ltsc", "i686") => iso_download_links.get(2),
        ("10-ltsc", "x86_64") => iso_download_links.get(3),
        (_, "i686") => iso_download_links.get(0),
        (_, "x86_64") => iso_download_links.get(1),
        (_, _) => return Err("Invalid architecture for provided release".into()),
    }.unwrap_or(iso_download_links.get(0).ok_or("Could not find download link")?);

    Ok((link.to_string(), "".to_string()))
}
