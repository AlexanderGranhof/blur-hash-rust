use core::f64;
use std::cmp;
use std::env;
use std::thread::JoinHandle;
use image::ImageBuffer;
use image::ImageReader;
use image::Rgb;
use std::io;
use std::io::Write;
use std::thread;
use std::sync::{Arc, Mutex};

struct BlurParams {
  data: ImageBuffer<Rgb<u8>, Vec<u8>>,
  width: u32,
  height: u32,
  x_component: u32,
  y_component: u32,
}


fn srgb_to_linear(value: f64) -> f64 {
  let v = value / 255.0;

  if v <= 0.04045 {
    return v / 12.92
  } else {
    return ((v + 0.055) / 1.055).powf(2.4)
  }
}


fn calc_factors(x: u32, y: u32, width: u32, height: u32, get_pixel: impl Fn(u32, u32) -> (u8,u8, u8)) -> (f64, f64, f64) {
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

          r += basis * srgb_to_linear(pixel.0 as f64);
          g += basis * srgb_to_linear(pixel.1 as f64);
          b += basis * srgb_to_linear(pixel.2 as f64);
        }
      }

      return (r * scale as f64, g * scale as f64, b * scale as f64);
}

struct ComputedFactor {
  y_component: u32,
  data: (f64, f64, f64),
}


fn blur(params: BlurParams) -> String {
  let clamped_width = params.width;
  let clamped_height = params.height;

  let arc_factors  = Arc::new(Mutex::<Vec<ComputedFactor>>::new(Vec::new()));

  let mut handles: Vec<JoinHandle<()>> = vec![];


  let raw = Arc::new(params.data);

  for y in 0..params.y_component {
    let local_factors = Arc::clone(&arc_factors);

    let other = Arc::clone(&raw);

    let handle = thread::spawn(move || {
      for x in 0..params.x_component {
        let factor = calc_factors(x, y, clamped_width, clamped_height, |i, j| {
          let pixel = other.get_pixel(i, j);

          return (
            pixel[0],
            pixel[1],
            pixel[2],
          )
        });

        let mut factors = local_factors.lock().unwrap();
  
        factors.push(ComputedFactor {
          y_component: y,
          data: factor,
        });
      }
    });

    handles.push(handle);
  }

  for handle in handles {
    handle.join().unwrap();
  }

  let mut factors = arc_factors.lock().unwrap();

  factors.sort_by_key(|factor| factor.y_component);

  let mapped_data = factors.iter().map(|factor| factor.data).collect::<Vec<(f64, f64, f64)>>();

  let dc = mapped_data[0];
  let ac = mapped_data[1..].to_vec();

  let mut hash: String = Default::default();

  let sizeflag = params.x_component - 1 + (params.y_component - 1) * 9;

  hash += &base83(sizeflag as f32, 1);

  let max: f64;

  if ac.len() > 0 {
    let mut real_max: f64 = 0.0;

    for rgb in &ac {
      for value in vec![rgb.0, rgb.1, rgb.2] {
        real_max = real_max.max(value);
      }
    }

    let d = real_max * 166.0 - 0.5;

    let quantised_max = cmp::max(0, cmp::min(82, d.floor() as i32));

    max = (quantised_max as f64 + 1.0) / 166.0;
    hash += &base83(quantised_max as f32, 1);
  } else {
    max = 1.0;
    hash += &base83(0.0, 1);
  }

  hash += &base83(encode_dc(dc) as f32, 4);


  for rgb in &ac {
    hash += &base83(encode_ac(&rgb, max) as f32, 2);
  }


  return hash;

}

fn main() {
  let args: Vec<String> = env::args().collect();
  
  let file_path = &args[1];

  let img = ImageReader::open(file_path).expect("faield 1").decode().expect("failed");

  let width = img.width();
  let height = img.height();


  let hash = blur(BlurParams {
    data: img.to_rgb8(),
    width,
    height,
    x_component: 4,
    y_component: 4,
  });

  let stdout = io::stdout();
  let mut w = io::BufWriter::new(stdout);

  writeln!(w, "{hash}").unwrap();
}

fn base83(n: f32, length: u32) -> String {
  let chars: &[&str] = &[
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "#",
    "$",
    "%",
    "*",
    "+",
    ",",
    "-",
    ".",
    ":",
    ";",
    "=",
    "?",
    "@",
    "[",
    "]",
    "^",
    "_",
    "{",
    "|",
    "}",
    "~",
  ];

  let mut result: String = Default::default();

  for i in 1..=length {
    let index = (n.floor() / (83.0 as f32).powi((length - i) as i32) % 83.0) as usize;

    result += chars.get(index).expect("could not find")
  }

  return result;
}

fn linear_to_srgb(value: f64) -> i32 {
  let v = (0.0 as f64).max(value.min(1.0));

  if v <= 0.0031308 {
    return (v * 12.92 * 255.0 + 0.5) as i32
  } else {
    return ((1.055 * v.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5) as i32
  }
}

fn encode_dc(value: (f64, f64, f64)) -> i32 {
  let rounded_r = linear_to_srgb(value.0);
  let rounded_g = linear_to_srgb(value.1);
  let rounded_b = linear_to_srgb(value.2);

  return rounded_r << 16 | rounded_g << 8 | rounded_b;
}

fn quantise (value: f64, max: f64) -> i32 {
  let pow = (sign_pow(value / max, 0.5) * 9.0 + 9.5) as i32;

  return 0.max(18.min(pow));
}

fn encode_ac(value: &(f64, f64, f64), max: f64) -> i32 {
  let quant_r = quantise(value.0, max);
  let quant_g = quantise(value.1, max);
  let quant_b = quantise(value.2, max);

  return quant_r * 19 * 19 + quant_g * 19 + quant_b;
}

fn sign (value: f64) -> i32 {
  if value < 0.0 {
    return -1;
  } else {
    return 1;
  }
}

fn sign_pow (value: f64, power: f64) -> f64 {
  return sign(value) as f64 * value.abs().powf(power);
}