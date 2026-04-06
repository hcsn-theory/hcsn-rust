use chrono::{Local};
use std::fs::{OpenOptions, File};
use std::io::{BufWriter, Write};
use crate::observables::InteractionEvent;

pub struct Persistence;

impl Persistence {
    /// Generates a filename with the format: exports/hcsn_<prefix>_YYYY-MM-DD_HH-MM-SS.csv
    pub fn generate_filename(prefix: &str) -> String {
        let now = Local::now();
        format!("exports/hcsn_{}_{}.csv", prefix, now.format("%Y-%m-%d_%H-%M-%S"))
    }

    /// Standard HCSN interaction header for scaling studies
    pub fn write_header(writer: &mut dyn Write) -> std::io::Result<()> {
        writeln!(writer, "pre_px,pre_py,post_px,post_py,pre_p_mag,post_p_mag,pre_mass,post_mass,pre_s_sum,post_s_sum,pre_s_mean,post_s_mean,chi,duration,stability_bin,pre_E_total,post_E_total")
    }

    /// Formats a single InteractionEvent into a CSV row
    pub fn format_event(event: &InteractionEvent) -> Option<String> {
        if event.duration < 3 { return None; }
        
        let (post_a, post_b) = match (event.post_a, event.post_b) {
            (Some(a), Some(b)) => (a, b),
            _ => return None,
        };

        let m_pre_a = event.pre_a.7 as f64 * event.pre_a.4.powi(2);
        let m_pre_b = event.pre_b.7 as f64 * event.pre_b.4.powi(2);
        let m_post_a = post_a.7 as f64 * post_a.4.powi(2);
        let m_post_b = post_b.7 as f64 * post_b.4.powi(2);
        
        let pre_px = (m_pre_a * event.pre_a.3.0) + (m_pre_b * event.pre_b.3.0);
        let pre_py = 0.0; 
        let post_px = (m_post_a * post_a.3.0) + (m_post_b * post_b.3.0);
        let post_py = 0.0;
        let pre_p_mag = (m_pre_a * event.pre_a.3.0).abs() + (m_pre_b * event.pre_b.3.0).abs();
        let post_p_mag = (m_post_a * post_a.3.0).abs() + (m_post_b * post_b.3.0).abs();
        let pre_s_sum = event.pre_a.5 + event.pre_b.5;
        let post_s_sum = post_a.5 + post_b.5;
        let pre_s_mean = (event.pre_a.5 + event.pre_b.5) / 2.0;
        let post_s_mean = (post_a.5 + post_b.5) / 2.0;
        let pre_E_total = event.pre_a.9 + event.pre_b.9;
        let post_E_total = post_a.9 + post_b.9;
        let stability_bin = (pre_s_mean / 5.0).floor() * 5.0;

        Some(format!("{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            pre_px, pre_py, post_px, post_py, pre_p_mag, post_p_mag, (m_pre_a + m_pre_b), (m_post_a + m_post_b),
            pre_s_sum, post_s_sum, pre_s_mean, post_s_mean, event.overlap_depth, event.duration as f64, stability_bin, pre_E_total, post_E_total))
    }

    /// Standard SSD persistence loop initializer
    pub fn open_writer(filename: &str) -> BufWriter<File> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(filename)
            .expect("Failed to open persistence file");
        BufWriter::new(file)
    }
}
