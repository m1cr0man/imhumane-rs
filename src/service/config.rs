#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub buffer_size: usize,

    pub image_size: u32,

    pub gap_size: u32,

    pub grid_length: u32,
}
