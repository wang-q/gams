use serde::{Deserialize, Serialize};

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
