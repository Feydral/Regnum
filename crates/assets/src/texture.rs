use std::{fs::File, io::BufReader, path::Path};

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Texture {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        assert_eq!(
            data.len(),
            width as usize * height as usize * 4,
            "texture data must be width * height RGBA8 pixels"
        );

        Self {
            width,
            height,
            data,
        }
    }

    pub fn load_png(path: impl AsRef<Path>) -> Result<Self, png::DecodingError> {
        let mut decoder = png::Decoder::new(BufReader::new(File::open(path)?));
        decoder.set_transformations(png::Transformations::normalize_to_color8());

        let mut reader = decoder.read_info()?;
        let mut buffer = vec![0; reader.output_buffer_size().unwrap_or(0)];
        let info = reader.next_frame(&mut buffer)?;
        let bytes = &buffer[..info.buffer_size()];

        let data = match info.color_type {
            png::ColorType::Rgba => bytes.to_vec(),
            png::ColorType::Rgb => bytes
                .chunks_exact(3)
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect(),
            png::ColorType::Grayscale => bytes.iter().flat_map(|&g| [g, g, g, 255]).collect(),
            png::ColorType::GrayscaleAlpha => bytes
                .chunks_exact(2)
                .flat_map(|p| [p[0], p[0], p[0], p[1]])
                .collect(),
            png::ColorType::Indexed => unreachable!("indexed images are expanded by the decoder"),
        };

        Ok(Self::new(info.width, info.height, data))
    }
}
