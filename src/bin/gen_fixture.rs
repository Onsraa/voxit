use std::io::BufWriter;
use std::path::Path;

use anyhow::Result;
use tiff::encoder::{colortype, TiffEncoder};
use tiff::tags::Tag;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const OUT_PATH: &str = "assets/test.tif";

const NODATA: i16 = -32768;

const PEAK_AMP_M: f64 = 1800.0;
const PEAK_CX: f64 = 64.0;
const PEAK_CY: f64 = 64.0;
const PEAK_SIGMA: f64 = 22.0;

const BASIN_AMP_M: f64 = -800.0;
const BASIN_CX: f64 = 32.0;
const BASIN_CY: f64 = 96.0;
const BASIN_SIGMA: f64 = 14.0;

const BASE_ELEV_M: f64 = 400.0;

const PIXEL_SCALE_DEG: f64 = 1.0 / 3600.0;
const ORIGIN_LON: f64 = 6.864;
const ORIGIN_LAT: f64 = 45.832;

fn elevation(x: f64, y: f64) -> f64 {
    let dxa = (x - PEAK_CX) / PEAK_SIGMA;
    let dya = (y - PEAK_CY) / PEAK_SIGMA;
    let peak = PEAK_AMP_M * (-0.5 * (dxa * dxa + dya * dya)).exp();

    let dxb = (x - BASIN_CX) / BASIN_SIGMA;
    let dyb = (y - BASIN_CY) / BASIN_SIGMA;
    let basin = BASIN_AMP_M * (-0.5 * (dxb * dxb + dyb * dyb)).exp();

    BASE_ELEV_M + peak + basin
}

fn main() -> Result<()> {
    let mut data: Vec<i16> = Vec::with_capacity((WIDTH * HEIGHT) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if x == 0 && y == 0 {
                data.push(NODATA);
                continue;
            }
            let z = elevation(x as f64, y as f64)
                .round()
                .clamp((i16::MIN as f64) + 1.0, i16::MAX as f64);
            data.push(z as i16);
        }
    }

    std::fs::create_dir_all("assets")?;
    let path = Path::new(OUT_PATH);
    let file = std::fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = TiffEncoder::new(writer)?;
    let mut image = encoder.new_image::<colortype::GrayI16>(WIDTH, HEIGHT)?;

    let scale: [f64; 3] = [PIXEL_SCALE_DEG, PIXEL_SCALE_DEG, 0.0];
    let tiepoint: [f64; 6] = [0.0, 0.0, 0.0, ORIGIN_LON, ORIGIN_LAT, 0.0];
    image
        .encoder()
        .write_tag(Tag::ModelPixelScaleTag, &scale[..])?;
    image
        .encoder()
        .write_tag(Tag::ModelTiepointTag, &tiepoint[..])?;

    image.write_data(&data)?;
    println!("wrote {} ({}x{} GrayI16)", OUT_PATH, WIDTH, HEIGHT);
    Ok(())
}
