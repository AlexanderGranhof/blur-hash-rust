use core::f64;
use std::cmp;
use std::io;
use std::io::Read;
use std::thread::JoinHandle;
use std::usize;
use base83::encode_base83;
use image::ImageBuffer;
use image::ImageReader;
use image::Rgb;
use image::Rgba;
use std::thread;
use std::sync::Arc;
use clap::Parser;

mod utils;
mod base83;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// x component of the hash (1-9)
  #[arg(short, default_value = "4")]
  x: u32,

  /// y component of the hash (1-9)
  #[arg(short, default_value = "4")]
  y: u32,

  /// step size for the sampling (1 = 100% of pixels, 2 = 50% of pixels, 10 = 10% of pixels etc...)
  #[arg(short, default_value = "1")]
  step: usize,

  /// Path to the image file
  filepath: String
}

fn validate(hash: &String) -> bool {
  if hash.len() != 6 {
    return false;
  }

  let char = match hash.chars().nth(0) {
    None => return false,
    Some(c) => c
  };

  let size_flag = base83::decode_base83(&String::from(char));
  let x = (size_flag % 9) + 1;
  let y = (size_flag as f32 / 9.0).floor() as u32 + 1;

  let valid_length = 4 + 2 * x * y;

  return hash.len() as u32 == valid_length;
}

fn decode(hash: String, width: u32, height: u32) -> Vec<Rgba<u8>> {
  let punch_value = 1;

  let size_flag = base83::decode_base83(&String::from(hash.chars().nth(0).unwrap()));
  let x_component = (size_flag % 9) + 1;
  let y_component = (size_flag as f32 / 9.0).floor() as u32 + 1;

  let quantised_max = base83::decode_base83(&hash.chars().nth(1).unwrap().to_string());
  let max = (quantised_max as f64 + 1.0) / 166.0;


  let size = x_component * y_component;
  let mut rgb_colors = vec![];

  for i in 0..size {
    if i == 0 {
      let value = base83::decode_base83(&String::from(&hash[2..6]));
      rgb_colors.push(utils::decode_dc(value));
    } else {
      let start = 4 + i as usize * 2;
      let end = 6 + i as usize * 2;

      let value = base83::decode_base83(&String::from(&hash[start..end]));
      rgb_colors.push(utils::decode_ac(value, max * (punch_value as f64)));
    }
  }

  let bytes_per_row = width * 4;
  let data_size = bytes_per_row * height;

  let mut rgb_data = vec![];

  for y in 0..height {
    for x in 0..width {
      let mut r = 0.0;
      let mut g = 0.0;
      let mut b = 0.0;

      for j in 0..y_component {
        let basis_y = f64::cos((f64::consts::PI * y as f64 * j as f64) / height as f64);

        for i in 0..x_component {
          let basis = f64::cos((f64::consts::PI * x as f64 * i as f64) / height as f64) * basis_y;
          let index = (i + j * x_component) as usize;
          let color = rgb_colors.get(index).unwrap();

          r += (color[0] * basis);
          g += (color[1] * basis);
          b += (color[2] * basis);
        }
      }
      
      let int_r = utils::linear_to_srgb(r as f64);
      let int_g = utils::linear_to_srgb(g as f64);
      let int_b = utils::linear_to_srgb(b as f64);

      rgb_data.push(Rgba([int_r as u8, int_g as u8, int_b as u8, 255]));
    }
  }

  return rgb_data;
}


fn main() {
  let args = Args::parse();

  let stdin = io::stdin();
  let mut stdin = stdin.lock(); // locking is optional

  let mut full_line = String::new();

  // Could also `match` on the `Result` if you wanted to handle `Err` 
  while let Ok(n_bytes) = stdin.read_to_string(&mut full_line) {
      if n_bytes == 0 { break }
  }

  println!("{}", full_line);

  let x_size = 512;
  let y_size: u32 = 512;

  let data = decode(full_line, x_size, y_size);

  let mut buffer = image::ImageBuffer::new(x_size,y_size);

  for x in 0..x_size {
    for y in 0..y_size {
      buffer.put_pixel(x, y, data[(y * y_size + x) as usize]);
    }
  }

  buffer.save("out.png").unwrap();

  std::process::exit(0);

  let image_file = match ImageReader::open(args.filepath) {
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
    x_component: args.x,
    y_component: args.y,
    sample_rate: args.step,
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

  print!("{}", hash);
}

enum EncodingError {
  InvalidComponentLength,
  UnknownThreadFailure,
}

struct BlurParams {
  data: ImageBuffer<Rgb<u8>, Vec<u8>>,
  width: u32,
  height: u32,
  x_component: u32,
  y_component: u32,
  sample_rate: usize,
}

struct ComputedFactor {
  y_component: u32,
  data: Rgb<f64>,
}

fn calc_blur_hash(BlurParams { x_component ,y_component, width, height, sample_rate, data }: BlurParams) -> Result<String, EncodingError> {
  if x_component < 1 || x_component > 9 || y_component < 1 || y_component > 9 {
    return Err(EncodingError::InvalidComponentLength);
  }

  let mut handles: Vec<JoinHandle<Vec<ComputedFactor>>> = vec![];

  let rgb_data = Arc::new(data);

  for y in 0..y_component {
    let cloned_rgb_data = Arc::clone(&rgb_data);

    let handle = thread::spawn(move || {
      let mut factors = vec![];

      for x in 0..x_component {
        let factor = calc_factors(x, y, width, height, sample_rate, |i, j| {
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

  let mut factors: Vec<ComputedFactor> = vec![];

  for handle in handles {
    let mut factor =  match handle.join() {
      Ok(factor) => factor,
      Err(_) => {
        return Err(EncodingError::UnknownThreadFailure);
      }
    };

    factors.append(&mut factor);
  }

  factors.sort_by_key(|factor| factor.y_component);

  let factor_data = factors.iter().map(|factor| factor.data).collect::<Vec<Rgb<f64>>>();

  let dc = factor_data[0];
  let ac = factor_data[1..].to_vec();
  let sizeflag = x_component - 1 + (y_component - 1) * 9;

  Ok(reduce_hash(dc, ac, sizeflag))
}

fn calc_factors(x: u32, y: u32, width: u32, height: u32, sample_rate: usize, get_pixel: impl Fn(u32, u32) -> Rgb<u8>) -> Rgb<f64> {
  let a = (width as f32 / sample_rate as f32) * (height as f32 / sample_rate as f32);

  let scale = 1.0 / a;
  let normalisation = if x == 0 && y == 0 { 1 } else { 2 };

  let mut r = 0.0;
  let mut g = 0.0;
  let mut b = 0.0;


  for i in (0..width).step_by(sample_rate) {
    for j in (0..height).step_by(sample_rate) {
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

  hash += &base83::encode_base83(flag as f32, 1);

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
    hash += &base83::encode_base83(quantised_max as f32, 1);
  } else {
    max = 1.0;
    hash += &base83::encode_base83(0.0, 1);
  }

  hash += &base83::encode_base83(utils::encode_dc(dc) as f32, 4);

  for rgb in &ac {
    hash += &base83::encode_base83(utils::encode_ac(&rgb, max) as f32, 2);
  }

  return hash;
}

