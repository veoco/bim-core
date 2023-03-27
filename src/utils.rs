const WIDTH: [(u32, u8); 38] = [
    (126, 1),
    (159, 0),
    (687, 1),
    (710, 0),
    (711, 1),
    (727, 0),
    (733, 1),
    (879, 0),
    (1154, 1),
    (1161, 0),
    (4347, 1),
    (4447, 2),
    (7467, 1),
    (7521, 0),
    (8369, 1),
    (8426, 0),
    (9000, 1),
    (9002, 2),
    (11021, 1),
    (12350, 2),
    (12351, 1),
    (12438, 2),
    (12442, 0),
    (19893, 2),
    (19967, 1),
    (55203, 2),
    (63743, 1),
    (64106, 2),
    (65039, 1),
    (65059, 0),
    (65131, 2),
    (65279, 1),
    (65376, 2),
    (65500, 1),
    (65510, 2),
    (120831, 1),
    (262141, 2),
    (1114109, 1),
];

fn get_width(o: u32) -> u8 {
    if o == 0xE || o == 0xF {
        return 0;
    }
    for (num, wid) in WIDTH {
        if o <= num {
            return wid;
        }
    }
    1
}

pub fn justify_name(name: &str, length: u8, left_right: bool) -> String {
    let mut name_width = 0;
    let mut justified_name = String::new();

    for c in name.chars() {
        let w = get_width(c as u32);
        if name_width + w < length {
            name_width += w;
            justified_name.push(c);
        }
    }

    if name_width < length {
        let space_count = length - name_width;
        let spaces = " ".repeat(space_count as usize);
        if left_right {
            justified_name += spaces.as_str();
        } else {
            justified_name = spaces + &justified_name;
        }
    }
    justified_name
}

pub struct SpeedTestResult {
    upload: f64,
    upload_status: String,
    download: f64,
    download_status: String,
    latency: f64,
    jitter: f64,
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
