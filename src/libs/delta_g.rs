//! `DeltaG` calculates deltaG of polymer DNA sequences
//!
//! # SYNOPSIS
//!
//! ```
//! use gars::DeltaG;
//!
//! let dg = DeltaG::new();
//! let seq = "TAACAAGCAATGAGATAGAGAAAGAAATATATCCA";
//! println!("deltaG is {}", dg.polymer(&seq.to_string()).unwrap());
//! ```
//!
//! # REFERENCE
//!  1. SantaLucia J, Jr. 2004. Annu Rev Biophys Biomol Struct;
//!  2. SantaLucia J, Jr. 1998. Proc Natl Acad Sci U S A;
//!
//! # ATTRIBUTES
//! `temp`: temperature, 37.0 degree centigrade
//! `salt`: salt concentration, Default: 1.0 (Na+), in M.
//!         Total sodium concentration should be above 0.05 M and below 1.1 M.
//! `dgnn`: free energy of each NN dimer

use bio::alphabets::dna;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct DeltaG {
    temp: f32,
    salt: f32,
    dgnn: HashMap<String, f32>,
}

impl DeltaG {
    // Immutable accessors
    pub fn temp(&self) -> &f32 {
        &self.temp
    }
    pub fn salt(&self) -> &f32 {
        &self.salt
    }

    pub fn dgnn(&self) -> &HashMap<String, f32> {
        &self.dgnn
    }

    /// Constructed from default temperature and salt concentration
    ///
    /// ```
    /// # use gars::DeltaG;
    /// let dg = DeltaG::new();
    /// assert_eq!(*dg.temp(), 37.0);
    /// assert_eq!(*dg.salt(), 1.0);
    /// ```
    pub fn new() -> Self {
        Self {
            temp: 37.0,
            salt: 1.0,
            dgnn: init_delta_g(37.0, 1.0),
        }
    }

    /// Constructed from temperature and salt concentration values
    ///
    /// ```
    /// # use gars::DeltaG;
    /// let dg = DeltaG::from(30.0, 0.5);
    /// # assert_eq!(*dg.temp(), 30.0);
    /// # assert_eq!(*dg.salt(), 0.5);
    /// ```
    pub fn from(temp: f32, salt: f32) -> Self {
        Self {
            temp,
            salt,
            dgnn: init_delta_g(temp, salt),
        }
    }

    pub fn polymer(&self, polymer: &String) -> Option<f32> {
        let seq = polymer.to_ascii_uppercase();

        let len = seq.len();
        if len < 3 {
            return None;
        }

        let nuc = ['A', 'G', 'C', 'T'];
        if !seq.chars().all(|c| nuc.contains(&c)) {
            return None;
        }

        let dgnn = self.dgnn();
        let len = seq.len();
        let mut dg = 0.0;

        // NN model, stop at the last second base
        for i in 0..(len - 1) {
            let nn = seq[i..(i + 2)].to_string();
            dg += dgnn.get(&nn).unwrap();
        }

        // terminal correction
        let init_ter = format!("i{}", seq.chars().nth(0).unwrap());
        dg += dgnn.get(&init_ter).unwrap();

        let end_ter = format!("i{}", seq.chars().nth(len - 1).unwrap());
        dg += dgnn.get(&end_ter).unwrap();

        // Symmetry correction
        let rc = dna::revcomp(seq.bytes());
        if seq.bytes().eq(rc) {
            dg += *dgnn.get("sym").unwrap();
        }

        Some(dg)
    }
}

fn init_delta_g(temp: f32, salt: f32) -> HashMap<String, f32> {
    // deltaH (kcal/mol)
    let delta_h: HashMap<String, f32> = [
        ("AA", -7.6),
        ("TT", -7.6),
        ("AT", -7.2),
        ("TA", -7.2),
        ("CA", -8.5),
        ("TG", -8.5),
        ("GT", -8.4),
        ("AC", -8.4),
        ("CT", -7.8),
        ("AG", -7.8),
        ("GA", -8.2),
        ("TC", -8.2),
        ("CG", -10.6),
        ("GC", -9.8),
        ("GG", -8.0),
        ("CC", -8.0),
        ("iC", 0.2),
        ("iG", 0.2),
        ("iA", 2.2),
        ("iT", 2.2),
        ("sym", 0.0),
    ]
    .iter()
    .cloned()
    .map(|t| (t.0.to_string(), t.1))
    .collect();

    // deltaS (cal/K.mol)
    let delta_s: HashMap<String, f32> = [
        ("AA", -21.3),
        ("TT", -21.3),
        ("AT", -20.4),
        ("TA", -21.3),
        ("CA", -22.7),
        ("TG", -22.7),
        ("GT", -22.4),
        ("AC", -22.4),
        ("CT", -21.0),
        ("AG", -21.0),
        ("GA", -22.2),
        ("TC", -22.2),
        ("CG", -27.2),
        ("GC", -24.4),
        ("GG", -19.9),
        ("CC", -19.9),
        ("iC", -5.7),
        ("iG", -5.7),
        ("iA", 6.9),
        ("iT", 6.9),
        ("sym", -1.4),
    ]
    .iter()
    .cloned()
    .map(|t| (t.0.to_string(), t.1))
    .collect();

    // deltaG
    // # dG = dH - TdS, and dS is dependent on the salt concentration
    let mut delta_g: HashMap<String, f32> = [
        ("iC", 1.96),
        ("iG", 1.96),
        ("iA", 0.05),
        ("iT", 0.05),
        ("sym", 0.43),
    ]
    .iter()
    .cloned()
    .map(|t| (t.0.to_string(), t.1))
    .collect();

    // the length of each NN dimer is 2, therefore the modifier is 1
    let entropy_adjust = 0.368 * salt.ln();

    for (key, value) in delta_h {
        // the length of each monomer is 1, thus the modifier of dS is 0
        // and the values are precalculated
        if key.starts_with("i") || key.starts_with("s") {
            continue;
        }

        let ds = delta_s.get(&key).unwrap() + entropy_adjust;
        let dg = value - ((273.15 + temp) * (ds / 1000.0));
        delta_g.insert(key, dg);
    }

    delta_g
}

#[test]
fn test_polymer() {
    let tests = vec![
        (
            37.0,
            1.0,
            "TAACAAGCAATGAGATAGAGAAAGAAATATATCCA".to_string(),
            Some(-39.2702),
        ),
        (
            30.0,
            0.1,
            "TAACAAGCAATGAGATAGAGAAAGAAATATATCCA".to_string(),
            Some(-35.6605),
        ),
        (37.0, 1.0, "GAATTC".to_string(), Some(-1.1399)),
        (37.0, 1.0, "NAATTC".to_string(), None),
    ];
    for (temp, salt, seq, expected) in tests {
        let dg = DeltaG::from(temp, salt);
        match expected {
            Some(res) => assert!((dg.polymer(&seq).unwrap() - res).abs() < 0.001),
            None => assert!(dg.polymer(&seq).is_none()),
        }
    }
}
