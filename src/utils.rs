use image::Rgb;

pub fn srgb_to_linear(value: f64) -> f64 {
  let v = value / 255.0;

  if v <= 0.04045 {
    return v / 12.92
  } else {
    return ((v + 0.055) / 1.055).powf(2.4)
  }
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

pub fn decode_dc(value: u32) -> Rgb<f64> {
  let r = srgb_to_linear((value >> 16) as f64);
  let g = srgb_to_linear(((value >> 8) & 255) as f64);
  let b = srgb_to_linear((value & 255) as f64);

  return Rgb([r, g, b]);
}

pub fn encode_ac(value: &Rgb<f64>, max: f64) -> i32 {
  let quant_r = quantise(value[0], max);
  let quant_g = quantise(value[1], max);
  let quant_b = quantise(value[2], max);

  return quant_r * 19 * 19 + quant_g * 19 + quant_b;
}

pub fn decode_ac(value: u32, max: f64) -> Rgb<f64> {
  let r = (value as f64 / (19.0 * 19.0)).floor() as u32;
  let g = ((value as f64 / 19.0).floor() as u32) % 19;
  let b = value % 19;

  return Rgb([
    sign_pow((r as f64 - 9.0) / 9.0, 2.0) * max,
    sign_pow((g as f64 - 9.0) / 9.0, 2.0) * max,
    sign_pow((b as f64 - 9.0) / 9.0, 2.0) * max,
  ]);
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