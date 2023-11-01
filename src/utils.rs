use std::fmt;

use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

pub fn justify_name(name: &str, length: u8, left_right: bool) -> String {
    let mut justified_name = String::from(name);
    let width = UnicodeWidthStr::width(name);

    if width < length as usize {
        let space_count = length as usize - width;
        let spaces = " ".repeat(space_count as usize);
        if left_right {
            justified_name += spaces.as_str();
        } else {
            justified_name = spaces + &justified_name;
        }
    }
    justified_name
}

#[derive(Serialize, Deserialize)]
pub struct SpeedTestResult {
    #[serde(serialize_with = "serialize_f64")]
    upload: f64,
    upload_status: String,
    #[serde(serialize_with = "serialize_f64")]
    download: f64,
    download_status: String,
    #[serde(serialize_with = "serialize_f64")]
    latency: f64,
    #[serde(serialize_with = "serialize_f64")]
    jitter: f64,
}

fn serialize_f64<S>(x: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = format!("{:.1}", x);
    serializer.serialize_str(&s)
}

impl SpeedTestResult {
    pub fn build(
        upload: f64,
        upload_status: String,
        download: f64,
        download_status: String,
        latency: f64,
        jitter: f64,
    ) -> SpeedTestResult {
        return SpeedTestResult {
            upload,
            upload_status,
            download,
            download_status,
            latency,
            jitter,
        };
    }

    pub fn text(&self) -> String {
        let upload = justify_name(&format!("{:.1}", &self.upload), 9, false);
        let upload_status = justify_name(&self.upload_status, 5, false);
        let download = justify_name(&format!("{:.1}", &self.download), 9, false);
        let download_status = justify_name(&self.download_status, 5, false);
        let latency = justify_name(&format!("{:.1}", &self.latency), 7, false);
        let jitter = justify_name(&format!("{:.1}", &self.jitter), 7, false);

        format!("{upload},{upload_status},{download},{download_status},{latency},{jitter}")
    }
}

impl fmt::Display for SpeedTestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Upload {:.1}Mbps {}, Download: {:.1}Mbps {}, Latency {:.1}, Jitter {:.1}",
            self.upload,
            self.upload_status,
            self.download,
            self.download_status,
            self.latency,
            self.jitter
        )
    }
}
