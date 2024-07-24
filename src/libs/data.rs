use serde::{Deserialize, Serialize};
use std::fmt;

/// chr_* fields were retained to facilitate Serde serializing to tsv
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ctg {
    pub id: String,
    pub range: String,
    pub chr_id: String,
    pub chr_start: i32,
    pub chr_end: i32,
    pub chr_strand: String,
    pub length: i32,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub range: String,
    pub length: i32,
    pub tag: String,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rg {
    pub id: String,
    pub range: String,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Peak {
    pub id: String,
    pub range: String,
    pub length: i32,
    pub gc: f32,
    pub signal: String,
    pub left_wave_length: Option<i32>,
    pub left_amplitude: Option<f32>,
    pub left_signal: Option<String>,
    pub right_wave_length: Option<i32>,
    pub right_amplitude: Option<f32>,
    pub right_signal: Option<String>,
}

#[derive(Default, Clone, Debug)]
pub struct Sw {
    pub id: String,
    pub range: String,
    pub sw_type: String,
    pub distance: i32,
    pub gc_content: Option<f32>,
    pub gc_mean: Option<f32>,
    pub gc_stddev: Option<f32>,
    pub gc_cv: Option<f32>,
    pub rg_count: Option<i32>,
}

impl fmt::Display for Sw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res_gc = if self.gc_content.is_some() {
            format!(
                "{}\t{}\t{}\t{}",
                self.gc_content.unwrap(),
                self.gc_mean.unwrap(),
                self.gc_stddev.unwrap(),
                self.gc_cv.unwrap()
            )
        } else {
            "\t\t\t".to_string() // empty fields
        };
        let res_rg = if self.rg_count.is_some() {
            format!("{}", self.rg_count.unwrap())
        } else {
            "".to_string()
        };
        write!(
            f,
            "{}\t{}\t{}\t{}\t{}\t{}",
            self.id, self.range, self.sw_type, self.distance, res_gc, res_rg
        )?;
        Ok(())
    }
}
