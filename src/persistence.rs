use crate::observables::InteractionEvent;
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};

pub struct Persistence;

impl Persistence {
    /// Generates a filename with the format: exports/hcsn_<prefix>_YYYY-MM-DD_HH-MM-SS.csv
    pub fn generate_filename(prefix: &str) -> String {
        let now = Local::now();
        format!(
            "exports/hcsn_{}_{}.csv",
            prefix,
            now.format("%Y-%m-%d_%H-%M-%S")
        )
    }

    /// Standard HCSN interaction header for scaling studies
    pub fn write_header(writer: &mut dyn Write) -> std::io::Result<()> {
        writeln!(writer, "thread_id,pre_px,pre_py,post_px,post_py,pre_p_mag,post_p_mag,pre_mass,post_mass,pre_s_sum,post_s_sum,pre_s_mean,post_s_mean,pre_age_a,pre_age_b,chi,duration,theta,stability_bin,pre_E_total,post_E_total")
    }

    /// Formats a single InteractionEvent into a CSV row
    pub fn format_event(event: &InteractionEvent, tid: usize) -> Option<String> {
        // We capture all interactions that lasted at least 1 cycle to recover the full statistical tail.
        if event.duration < 1 {
            return None;
        }

        let m_pre_a = event.pre_a.7 as f64 * event.pre_a.4.powi(2);
        let m_pre_b = event.pre_b.7 as f64 * event.pre_b.4.powi(2);
        let vx_pre_a = event.pre_a.3 .0.clamp(-10.0, 10.0);
        let vx_pre_b = event.pre_b.3 .0.clamp(-10.0, 10.0);
        let age_a = event.pre_a.10;
        let age_b = event.pre_b.10;

        let (m_post_a, vx_post_a, s_post_a) = if let Some(a) = event.post_a {
            (a.7 as f64 * a.4.powi(2), a.3 .0.clamp(-10.0, 10.0), a.5)
        } else {
            (0.0, 0.0, 0.0) // Destruction state
        };

        let (m_post_b, vx_post_b, s_post_b) = if let Some(b) = event.post_b {
            (b.7 as f64 * b.4.powi(2), b.3 .0.clamp(-10.0, 10.0), b.5)
        } else {
            (0.0, 0.0, 0.0) // Destruction state
        };

        let pre_px = (m_pre_a * vx_pre_a) + (m_pre_b * vx_pre_b);
        let pre_py = 0.0;
        let post_px = (m_post_a * vx_post_a) + (m_post_b * vx_post_b);
        let post_py = 0.0;
        let pre_p_mag = (m_pre_a * vx_pre_a).abs() + (m_pre_b * vx_pre_b).abs();
        let post_p_mag = (m_post_a * vx_post_a).abs() + (m_post_b * vx_post_b).abs();

        let pre_s_mean = (event.pre_a.5 + event.pre_b.5) / 2.0;
        let post_s_mean = (s_post_a + s_post_b) / 2.0;
        let pre_s_sum = event.pre_a.5 + event.pre_b.5;
        let post_s_sum = s_post_a + s_post_b;

        let pre_e_total = event.pre_a.9 + event.pre_b.9;
        let post_e_total = if let (Some(a), Some(b)) = (event.post_a, event.post_b) {
            a.9 + b.9
        } else {
            0.0
        };
        let stability_bin = (pre_s_mean / 5.0).floor() * 5.0;

        // Numerical Integrity Check: Detect and discard NaN/Inf events
        if !pre_p_mag.is_finite() || !post_p_mag.is_finite() || !pre_s_mean.is_finite() {
            return None;
        }

        Some(format!("{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            tid, pre_px, pre_py, post_px, post_py, pre_p_mag, post_p_mag, (m_pre_a + m_pre_b), (m_post_a + m_post_b),
            pre_s_sum, post_s_sum, pre_s_mean, post_s_mean, age_a, age_b, event.overlap_depth, event.duration as f64, 0.0, stability_bin, pre_e_total, post_e_total))
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
