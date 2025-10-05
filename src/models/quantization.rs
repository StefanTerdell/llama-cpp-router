#![allow(clippy::enum_variant_names)]

use std::fmt::Display;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub enum Q2 {
    #[serde(rename = "IQ2_XXS")]
    IQ2XXS,
    #[serde(rename = "IQ2_M")]
    IQ2M,
    #[serde(rename = "Q2_K")]
    #[default]
    Q2K,
    #[serde(rename = "Q2_K_L")]
    Q2KL,
    #[serde(rename = "Q2_K_XL")]
    Q2KXL,
}

impl Q2 {
    pub const BITS_PER_PARAM: u8 = 2;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub enum Q3 {
    #[serde(rename = "IQ3_XXS")]
    IQ3XXS,
    #[serde(rename = "Q3_K_S")]
    Q3KS,
    #[serde(rename = "Q3_K_M")]
    #[default]
    Q3KM,
    #[serde(rename = "Q3_K_XL")]
    Q3KXL,
}

impl Q3 {
    pub const BITS_PER_PARAM: u8 = 3;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub enum Q4 {
    #[serde(rename = "IQ4_XS")]
    IQ4XS,
    #[serde(rename = "IQ4_NL")]
    IQ4NL,
    #[serde(rename = "Q4_K_S")]
    Q4KS,
    #[serde(rename = "Q4_K_M")]
    #[default]
    Q4KM,
    #[serde(rename = "Q4_K_XL")]
    Q4KXL,
    #[serde(rename = "Q4_0")]
    Q40,
    #[serde(rename = "Q4_1")]
    Q41,
}

impl Q4 {
    pub const BITS_PER_PARAM: u8 = 4;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub enum Q5 {
    // 5-bit
    #[serde(rename = "Q5_K_S")]
    Q5KS,
    #[serde(rename = "Q5_K_M")]
    #[default]
    Q5KM,
    #[serde(rename = "Q5_K_XL")]
    Q5KXL,
}

impl Q5 {
    pub const BITS_PER_PARAM: u8 = 5;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub enum Q6 {
    // 6-bit
    #[serde(rename = "Q6_K")]
    #[default]
    Q6K,
    #[serde(rename = "Q6_K_XL")]
    Q6KXL,
}

impl Q6 {
    pub const BITS_PER_PARAM: u8 = 6;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub enum Q8 {
    // 8-bit
    #[serde(rename = "Q8_0")]
    #[default]
    Q80,
    #[serde(rename = "Q8_K_XL")]
    Q8KXL,
}

impl Q8 {
    pub const BITS_PER_PARAM: u8 = 8;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum Quantization {
    #[serde(rename = "BF16")]
    BF16,
    #[serde(untagged)]
    Q2(Q2),
    #[serde(untagged)]
    Q3(Q3),
    #[serde(untagged)]
    Q4(Q4),
    #[serde(untagged)]
    Q5(Q5),
    #[serde(untagged)]
    Q6(Q6),
    #[serde(untagged)]
    Q8(Q8),
}

impl Quantization {
    pub fn bits_per_param(&self) -> u8 {
        match self {
            Quantization::Q2(_) => Q2::BITS_PER_PARAM,
            Quantization::Q3(_) => Q3::BITS_PER_PARAM,
            Quantization::Q4(_) => Q4::BITS_PER_PARAM,
            Quantization::Q5(_) => Q5::BITS_PER_PARAM,
            Quantization::Q6(_) => Q6::BITS_PER_PARAM,
            Quantization::Q8(_) => Q8::BITS_PER_PARAM,
            Quantization::BF16 => 16,
        }
    }

    pub fn bytes_per_param(&self) -> f32 {
        self.bits_per_param() as f32 / 8.0
    }

    pub fn compression_ratio(&self) -> f32 {
        16.0 / self.bits_per_param() as f32
    }
}

impl Display for Q6 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(serde_json::to_string(self).unwrap().trim_matches('"'))
    }
}
