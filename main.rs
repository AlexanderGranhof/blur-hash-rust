use core::f64;
use std::cmp;
use std::convert::Infallible;
use std::env;
use std::ops::Index;
use image::DynamicImage;
use image::GenericImageView;
use image::ImageBuffer;
// use std::io::Cursor;
use image::ImageReader;
use image::ImageError;
use image::Pixel;
use image::Pixels;
use image::Rgb;
use image::Rgba;
use std::io;
use std::io::Write;

struct BlurParams {
  data: ImageBuffer<Rgb<u8>, Vec<u8>>,
  width: u32,
  height: u32,
  xComponent: u32,
  yComponent: u32,
}

const BYTES_PER_PIXEL: u32 = 4;

fn srgb_to_linear(value: f64) -> f64 {
  let v = value / 255.0;

  if v <= 0.04045 {
    return v / 12.92
  } else {
    return ((v + 0.055) / 1.055).powf(2.4)
  }
}

struct ClampedValues {
  width: u32,
  height: u32,
}

fn clamp(width: u32, height: u32, max: u32) -> ClampedValues {
  if width >= height && width > max {
    return ClampedValues {
      width: max,
      height: ((height as f64 / width as f64) * max as f64).floor() as u32,
    };
  }

  if height > width && height > max {
    return ClampedValues {
      width: ((width as f64 / height as f64) * max as f64).floor() as u32,
      height: max,
    };
  }

  return ClampedValues {
    width,
    height,
  };
} 


fn blur(params: BlurParams) -> String {
  let clamped_width = params.width;
  let clamped_height = params.height;

  let mut factors: Vec<(f64, f64, f64)> = Vec::new();
  let scale = 1.0 / (clamped_width as f32 * clamped_height as f32);


  for y in 0..params.yComponent {
    for x in 0..params.xComponent {
      let normalisation = if x == 0 && y == 0 { 1 } else { 2 };

      let mut r = 0.0;
      let mut g = 0.0;
      let mut b = 0.0;

      for i in 0..clamped_width {
        for j in 0..clamped_height {
          let w = (f64::consts::PI * x as f64 * i as f64 ) / clamped_width as f64;
          let h = (f64::consts::PI * y as f64 * j as f64) / clamped_height as f64;

          let basis = normalisation as f64 * w.cos() * h.cos();

          let pixel = params.data.get_pixel(i, j);


          let index_r = 0;
          let index_g = 1;
          let index_b = 2;

          r += basis * srgb_to_linear(pixel[index_r] as f64);
          g += basis * srgb_to_linear(pixel[index_g] as f64);
          b += basis * srgb_to_linear(pixel[index_b] as f64);
        }
      }

      factors.push((r * scale as f64, g * scale as f64, b * scale as f64));
    }
  }

  let dc = factors[0];
  let ac = factors[1..].to_vec();

  let mut hash: String = Default::default();

  let sizeflag = params.xComponent - 1 + (params.yComponent - 1) * 9;

  hash += &base83(sizeflag as f32, 1);

  let mut max = 0.0;

  if ac.len() > 0 {
    let mut actualMax: f64 = 0.0;

    for rgb in &ac {
      for value in vec![rgb.0, rgb.1, rgb.2] {
        actualMax = actualMax.max(value);
      }
    }

    let d = actualMax * 166.0 - 0.5;

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
    xComponent: 4,
    yComponent: 4,
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
  let roundedR = linear_to_srgb(value.0);
  let roundedG = linear_to_srgb(value.1);
  let roundedB = linear_to_srgb(value.2);

  return roundedR << 16 | roundedG << 8 | roundedB;
}

fn quantise (value: f64, max: f64) -> i32 {
  let pow = (signPow(value / max, 0.5) * 9.0 + 9.5) as i32;

  return 0.max(18.min(pow));
}

fn encode_ac(value: &(f64, f64, f64), max: f64) -> i32 {
  let quantR = quantise(value.0, max);
  let quantG = quantise(value.1, max);
  let quantB = quantise(value.2, max);

  return quantR * 19 * 19 + quantG * 19 + quantB;
}

fn sign (value: f64) -> i32 {
  if value < 0.0 {
    return -1;
  } else {
    return 1;
  }
}

fn signPow (value: f64, power: f64) -> f64 {
  return sign(value) as f64 * value.abs().powf(power);
}