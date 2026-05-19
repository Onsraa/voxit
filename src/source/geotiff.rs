use std::path::Path;

use anyhow::{anyhow, Context, Result};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

use super::constants::{BIOME_PALETTE, GEOTIFF_NODATA_SRTM};
use super::{Palette, RawVolume, SourceKind, ThresholdConfig, VolumeSource};

pub struct GeoTiffSource;

impl VolumeSource for GeoTiffSource {
    fn parse(path: &Path) -> Result<RawVolume> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("open {}", path.display()))?;
        let reader = std::io::BufReader::new(file);
        let mut decoder = Decoder::new(reader).context("create tiff decoder")?;

        let (width, height) = decoder.dimensions().context("read tiff dimensions")?;
        let pixel_scale = decoder
            .get_tag_f64_vec(Tag::ModelPixelScaleTag)
            .context("ModelPixelScaleTag missing or non-numeric")?;
        let tiepoint = decoder
            .get_tag_f64_vec(Tag::ModelTiepointTag)
            .context("ModelTiepointTag missing or non-numeric")?;

        let image = decoder.read_image().context("read tiff image data")?;
        let expected = (width as usize)
            .checked_mul(height as usize)
            .ok_or_else(|| anyhow!("pixel count overflow"))?;
        let mut data = decoding_result_to_f32(image, expected)?;

        for v in data.iter_mut() {
            if *v == GEOTIFF_NODATA_SRTM {
                *v = f32::NAN;
            }
        }

        let spacing = [
            pixel_scale.first().copied().unwrap_or(1.0) as f32,
            pixel_scale.get(1).copied().unwrap_or(1.0) as f32,
            1.0,
        ];
        let origin = [
            tiepoint.get(3).copied().unwrap_or(0.0) as f32,
            tiepoint.get(4).copied().unwrap_or(0.0) as f32,
            tiepoint.get(5).copied().unwrap_or(0.0) as f32,
        ];

        Ok(RawVolume {
            data,
            dims: [width, height, 1],
            spacing,
            origin,
            source_kind: SourceKind::GeoTiff,
        })
    }

    fn default_thresholds(volume: &RawVolume) -> ThresholdConfig {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for &v in &volume.data {
            if v.is_finite() {
                if v < min {
                    min = v;
                }
                if v > max {
                    max = v;
                }
            }
        }
        ThresholdConfig { min, max }
    }

    fn palette_preset() -> Palette {
        Palette {
            name: "geotiff-biome-default",
            colors: BIOME_PALETTE.to_vec(),
        }
    }
}

fn decoding_result_to_f32(result: DecodingResult, expected: usize) -> Result<Vec<f32>> {
    let v: Vec<f32> = match result {
        DecodingResult::U8(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::U16(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::U32(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::U64(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::I8(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::I16(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::I32(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::I64(v) => v.into_iter().map(|x| x as f32).collect(),
        DecodingResult::F32(v) => v,
        DecodingResult::F64(v) => v.into_iter().map(|x| x as f32).collect(),
    };
    if v.len() != expected {
        anyhow::bail!("pixel count {} != expected {}", v.len(), expected);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;
    use tiff::encoder::{colortype, TiffEncoder};

    fn write_synthetic(path: &Path, width: u32, height: u32) -> Result<()> {
        let mut data: Vec<i16> = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                if x == 0 && y == 0 {
                    data.push(-32768);
                } else {
                    data.push((x as i16) * 10 + (y as i16));
                }
            }
        }

        let file = std::fs::File::create(path)?;
        let writer = BufWriter::new(file);
        let mut encoder = TiffEncoder::new(writer)?;
        let mut image = encoder.new_image::<colortype::GrayI16>(width, height)?;

        let scale: [f64; 3] = [0.001, 0.001, 0.0];
        let tiepoint: [f64; 6] = [0.0, 0.0, 0.0, 7.5, 45.5, 0.0];
        image
            .encoder()
            .write_tag(Tag::ModelPixelScaleTag, &scale[..])?;
        image
            .encoder()
            .write_tag(Tag::ModelTiepointTag, &tiepoint[..])?;

        image.write_data(&data)?;
        Ok(())
    }

    fn temp_path(label: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "voxel_viewer_geotiff_{}_{}.tif",
            label,
            std::process::id()
        ));
        p
    }

    #[test]
    fn parses_synthetic_geotiff_roundtrip() {
        let tmp = temp_path("roundtrip");
        write_synthetic(&tmp, 8, 6).expect("write synthetic tif");

        let volume = GeoTiffSource::parse(&tmp).expect("parse synthetic tif");
        let _ = std::fs::remove_file(&tmp);

        assert_eq!(volume.dims, [8, 6, 1]);
        assert!((volume.spacing[0] - 0.001).abs() < 1e-9);
        assert!((volume.spacing[1] - 0.001).abs() < 1e-9);
        assert!((volume.origin[0] - 7.5).abs() < 1e-6);
        assert!((volume.origin[1] - 45.5).abs() < 1e-6);

        assert!(volume.data[0].is_nan(), "(0,0) NODATA → NaN");
        assert_eq!(volume.data[1], 10.0, "(1,0) → 10");
        assert_eq!(volume.data[5 * 8 + 7], 75.0, "(7,5) → 75");
    }

    #[test]
    fn default_thresholds_skip_nan() {
        let volume = RawVolume {
            data: vec![10.0, 5.0, f32::NAN, 100.0, -3.0],
            dims: [5, 1, 1],
            spacing: [1.0, 1.0, 1.0],
            origin: [0.0, 0.0, 0.0],
            source_kind: SourceKind::GeoTiff,
        };
        let t = GeoTiffSource::default_thresholds(&volume);
        assert_eq!(t.min, -3.0);
        assert_eq!(t.max, 100.0);
    }
}
