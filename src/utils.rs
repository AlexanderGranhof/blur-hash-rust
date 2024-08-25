use image::Rgb;

pub fn srgb_to_linear(value: f64) -> f64 {
  let v = value / 255.0;

  if v <= 0.04045 {
    return v / 12.92
  } else {
    return ((v + 0.055) / 1.055).powf(2.4)
  }
}

pub fn base83(n: f32, length: u32) -> String {
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


pub fn linear_to_srgb(value: f64) -> i32 {
  let v = (0.0 as f64).max(value.min(1.0));

  if v <= 0.0031308 {
    return (v * 12.92 * 255.0 + 0.5) as i32
  } else {
    return ((1.055 * v.powf(1.0 / 2.4) - 0.055) * 255.0 + 0.5) as i32
  }
}

pub fn encode_dc(value: Rgb<f64>) -> i32 {
  let rounded_r = linear_to_srgb(value[0]);
  let rounded_g = linear_to_srgb(value[1]);
  let rounded_b = linear_to_srgb(value[2]);

  return rounded_r << 16 | rounded_g << 8 | rounded_b;
}

pub fn encode_ac(value: &Rgb<f64>, max: f64) -> i32 {
  let quant_r = quantise(value[0], max);
  let quant_g = quantise(value[1], max);
  let quant_b = quantise(value[2], max);

  return quant_r * 19 * 19 + quant_g * 19 + quant_b;
}

fn quantise (value: f64, max: f64) -> i32 {
  let pow = (sign_pow(value / max, 0.5) * 9.0 + 9.5) as i32;

  return 0.max(18.min(pow));
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