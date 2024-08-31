use std::ops::Index;

const BASE83_CHARS: [char; 83] = [
    '0',
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    'A',
    'B',
    'C',
    'D',
    'E',
    'F',
    'G',
    'H',
    'I',
    'J',
    'K',
    'L',
    'M',
    'N',
    'O',
    'P',
    'Q',
    'R',
    'S',
    'T',
    'U',
    'V',
    'W',
    'X',
    'Y',
    'Z',
    'a',
    'b',
    'c',
    'd',
    'e',
    'f',
    'g',
    'h',
    'i',
    'j',
    'k',
    'l',
    'm',
    'n',
    'o',
    'p',
    'q',
    'r',
    's',
    't',
    'u',
    'v',
    'w',
    'x',
    'y',
    'z',
    '#',
    '$',
    '%',
    '*',
    '+',
    ',',
    '-',
    '.',
    ':',
    ';',
    '=',
    '?',
    '@',
    '[',
    ']',
    '^',
    '_',
    '{',
    '|',
    '}',
    '~',
  ];

pub fn encode_base83(n: f32, length: u32) -> String {
  let mut result: String = Default::default();

  for i in 1..=length {
    let index = (n.floor() / (83.0 as f32).powi((length - i) as i32) % 83.0) as usize;

    result += &BASE83_CHARS.index(index).to_string();
  }

  return result;
}

pub fn decode_base83(str: &String) -> u32 {
  let mut value: u32 = 0;

  for char in str.chars() {
    let index = &BASE83_CHARS.iter().position(|&c| c == char).unwrap();
    value = value * 83 + *index as u32;
  }

  return value;
}