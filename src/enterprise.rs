use std::error::Error;

pub fn get_enterprise_info(release: &str, lang: &str, arch: &str)-> Result<(String, Option<String>), Box<dyn Error>> {
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

    let enterprise_lang = match lang {
        "English (Great Britain)" => "&culture=en-gb&country=GB",
        "Chinese (Simplified)" => "&culture=zh-cn&country=CN",
        "Chinese (Traditional)" => "&culture=zh-tw&country=TW",
        "French" => "&culture=fr-fr&country=FR",
        "German" => "&culture=de-de&country=DE",
        "Italian" => "&culture=it-it&country=IT",
        "Japanese" => "&culture=ja-jp&country=JP",
        "Korean" => "&culture=ko-kr&country=KR",
        "Portuguese (Brazil)" => "&culture=pt-br&country=BR",
        "Spanish" => "&culture=es-es&country=ES",
        "Russian" => "&culture=ru-ru&country=RU",
        _ => "&culture=en-us&country=US",
    };

    let download_page_html = download_page_html.replace("&amp;", "&");
    let iso_download_links = download_page_html.split("https://go.microsoft.com/").filter_map(|line| {
        if line.contains(enterprise_lang) {
            Some(line.split_inclusive(enterprise_lang).next().unwrap())
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

    Ok(("https://go.microsoft.com".to_owned() + link, None))
}
