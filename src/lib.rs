use std::error::Error;

#[cfg(feature = "consumer")]
mod consumer;
#[cfg(feature = "enterprise")]
mod enterprise;

pub struct WindowsRelease {
    pub release: String,
    pub architecture: String,
    pub lang: String,
    pub url: String,
    pub hash: String,
}

impl WindowsRelease {
    pub fn new(release: &str, lang: &str, arch: &str) -> Result<Self, Box<dyn Error>> {
        let data: (String, String) = match release {
            #[cfg(feature = "consumer")]
            "8" | "10" | "11" => {
                consumer::get_consumer_info(release, lang, arch)?
            },
            #[cfg(feature = "enterprise")]
            "10-enterprise" | "10-ltsc" | "11-enterprise" | "server-2012-r2" | "server-2016" | "server-2019" | "server-2022" => {
                enterprise::get_enterprise_info(release, lang, arch)?
            },
            _ => return Err("Unsupported release".into()),
        };
        Ok(Self {
            release: release.to_string(),
            architecture: arch.to_string(),
            lang: lang.to_string(),
            url: data.0,
            hash: data.1,
        })
    }
}
