use core::f64;
use std::cmp;
use std::env;
use std::thread::JoinHandle;
use image::ImageBuffer;
use image::ImageReader;
use image::Rgb;
use std::thread;
use std::sync::{Arc, Mutex};

mod utils;

fn main() {
  let args: Vec<String> = env::args().collect();
  let file_path = &args[1];

  let image_file = match ImageReader::open(file_path) {
    Ok(img) => img,
    Err(error) => {
      eprint!("Could not open image file: {error:?}");
      std::process::exit(1);
    }
  };

  let img = match image_file.decode() {
    Ok(img) => img,
    Err(e) => {
      eprint!("Could not decode image: {e:?}");
      std::process::exit(1);
    },
  };

  let width = img.width();
  let height = img.height();

  let hash = match calc_blur_hash(BlurParams {
    data: img.to_rgb8(),
    width,
    height,
    x_component: 4,
    y_component: 4,
  }) {
    Ok(hash) => hash,
    Err(error) => match error {
      EncodingError::InvalidComponentLength => {
        eprintln!("Invalid component length");
        std::process::exit(1);
      },

      _ => {
        eprintln!("Could not calculate blur hash");
        std::process::exit(1);
      }
    }
  };

  print!("{:?}", hash);
}

struct BlurParams {
  data: ImageBuffer<Rgb<u8>, Vec<u8>>,
  width: u32,
  height: u32,
  x_component: u32,
  y_component: u32,
}

struct ComputedFactor {
  y_component: u32,
  data: Rgb<f64>,
}

fn calc_factors(x: u32, y: u32, width: u32, height: u32, get_pixel: impl Fn(u32, u32) -> Rgb<u8>) -> Rgb<f64> {
  let scale = 1.0 / (width as f32 * height as f32);
  let normalisation = if x == 0 && y == 0 { 1 } else { 2 };

  let mut r = 0.0;
  let mut g = 0.0;
  let mut b = 0.0;

  for i in 0..width {
    for j in 0..height {
      let w = (f64::consts::PI * x as f64 * i as f64 ) / width as f64;
      let h = (f64::consts::PI * y as f64 * j as f64) / height as f64;

      let basis = normalisation as f64 * w.cos() * h.cos();
      let pixel = get_pixel(i, j);

      r += basis * utils::srgb_to_linear(pixel[0] as f64);
      g += basis * utils::srgb_to_linear(pixel[1] as f64);
      b += basis * utils::srgb_to_linear(pixel[2] as f64);
    }
  }

  return Rgb([
    r * scale as f64,
    g * scale as f64,
    b * scale as f64
  ]);
}

fn reduce_hash(dc: Rgb<f64>, ac: Vec<Rgb<f64>>, flag: u32) -> String {
  let mut hash: String = Default::default();
  let max: f64;

  hash += &utils::base83(flag as f32, 1);

  if ac.len() > 0 {
    let mut real_max: f64 = 0.0;

    for rgb in &ac {
      for value in vec![rgb[0], rgb[1], rgb[2]] {
        real_max = real_max.max(value);
      }
    }

    let adjusted_max: f64 = real_max * 166.0 - 0.5;
    let quantised_max = cmp::max(0, cmp::min(82, adjusted_max.floor() as i32));

    max = (quantised_max as f64 + 1.0) / 166.0;
    hash += &utils::base83(quantised_max as f32, 1);
  } else {
    max = 1.0;
    hash += &utils::base83(0.0, 1);
  }

  hash += &utils::base83(utils::encode_dc(dc) as f32, 4);

  for rgb in &ac {
    hash += &utils::base83(utils::encode_ac(&rgb, max) as f32, 2);
  }

  return hash;
}

enum EncodingError {
  InvalidComponentLength,
  UnknownThreadFailure,
}

fn calc_blur_hash(BlurParams { x_component ,y_component, width, height, data }: BlurParams) -> Result<String, EncodingError> {

  if x_component < 1 || x_component > 9 || y_component < 1 || y_component > 9 {
    return Err(EncodingError::InvalidComponentLength);
  }

  let mut handles: Vec<JoinHandle<Vec<ComputedFactor>>> = vec![];

  let arc_factors  = Arc::new(Mutex::<Vec<ComputedFactor>>::new(Vec::new()));
  let rgb_data = Arc::new(data);

  for y in 0..y_component {
    let cloned_rgb_data = Arc::clone(&rgb_data);

    let handle = thread::spawn(move || {
      let mut factors = vec![];

      for x in 0..x_component {
        let factor = calc_factors(x, y, width, height, |i, j| {
          return *cloned_rgb_data.get_pixel(i, j);
        });

        factors.push(ComputedFactor {
          y_component: y,
          data: factor,
        })
      }

      return factors;
    });

    handles.push(handle);
  }

  let mut factors = vec![];

  for handle in handles {
    let factor =  match handle.join() {
      Ok(factor) => factor,
      Err(_) => {
        return Err(EncodingError::UnknownThreadFailure);
      }
    };

    factors.push(factor);
  }

  let mut factors = arc_factors.lock().unwrap();

  factors.sort_by_key(|factor| factor.y_component);

  let factor_data = factors.iter().map(|factor| factor.data).collect::<Vec<Rgb<f64>>>();

  let dc = factor_data[0];
  let ac = factor_data[1..].to_vec();
  let sizeflag = x_component - 1 + (y_component - 1) * 9;

  Ok(reduce_hash(dc, ac, sizeflag))
}