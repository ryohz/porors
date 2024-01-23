use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::Read, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AaFont {
    width: usize,
    height: usize,
    margin: usize,
    chars: String,
    #[serde(skip_deserializing, skip_serializing)]
    art_chars: Vec<ArtString>,
    #[serde(skip_deserializing, skip_serializing)]
    fixed: bool,
}

impl AaFont {
    pub fn new(path: &PathBuf, fixed: bool) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("failed to open font file: {}", path.display()))?;
        let mut reader = std::io::BufReader::new(file);
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .with_context(|| format!("failed to read font file: {}", path.display()))?;

        let mut font = toml::from_str::<Self>(&buf).with_context(|| {
            format!("failed to parse toml file (font file): {}", path.display())
        })?;

        let mut art_chars = vec![];
        let lines = font.chars.split("\n").collect::<Vec<&str>>();
        for i in 0..(lines.len() - (font.height + font.margin) as usize - 1) {
            if i % (font.height + font.margin) as usize == 0 {
                let mut char_vec = vec![];
                for j in i..=(i + font.height - font.margin) {
                    let l = lines.get(j).unwrap().to_string();
                    let l = l.chars().take(font.width).collect::<String>();
                    char_vec.push(l);
                }

                let art_char = ArtString::from(font.width, font.height, char_vec, fixed);
                art_chars.push(art_char);
            }
        }

        font.art_chars = art_chars;
        Ok(font)
    }

    fn get_char(&self, ascii: u8) -> ArtString {
        if ascii < 0x21 {
            let mut space = vec![];
            let s = " ".repeat(self.width);
            for _ in 0..self.height {
                space.push(s.clone());
            }
            let space = ArtString::from(self.width, self.height, space, self.fixed);
            return space;
        }
        let index = ascii as usize - 0x21;
        let art_char = self.art_chars.get(index).unwrap();
        art_char.clone()
    }

    pub fn get_string(&self, string: &str) -> ArtString {
        let mut art_string = ArtString::new();
        for char in string.chars() {
            let code = char as u8;
            let art_char = self.get_char(code);
            art_string = art_string + art_char;
        }

        return art_string;
    }
}

#[derive(Debug, Clone)]
pub struct ArtString {
    length: usize,
    width: usize,
    height: usize,
    data: Vec<String>,
    fixed: bool,
}

impl ArtString {
    pub fn new() -> Self {
        Self {
            data: vec![],
            width: 0,
            height: 0,
            fixed: false,
            length: 0,
        }
    }

    fn from(width: usize, height: usize, data: Vec<String>, fixed: bool) -> Self {
        for d in &data {
            if d.chars().count() != width {
                panic!("the data size and given size are different");
            }
        }
        if data.len() != height {
            panic!("the data size and given size are different")
        }

        let data = data.iter().map(|x| x.to_string()).collect();

        Self {
            data,
            width,
            height,
            fixed,
            length: 1,
        }
    }

    fn connect(self, other: Self) -> Self {
        if self.length == 0 {
            return other;
        } else if other.length == 0 {
            return self;
        }

        let width = if self.width != other.width && (self.fixed || other.fixed) {
            panic!("failed to connect art characters - width is not same");
        } else if self.width <= other.width {
            self.width
        } else {
            other.width
        };

        let height = if self.height != other.height && (self.fixed || other.fixed) {
            panic!("failed to connect art characters - height is not same");
        } else if self.height <= other.height {
            self.height
        } else {
            other.height
        };

        let length = self.length + other.length;

        let mut data = vec![];

        for i in 0..height {
            let mut line = self.data[i].to_owned();
            let line2 = &other.data[i];

            line += line2;
            data.push(line);
        }

        Self {
            length,
            width,
            height,
            data,
            fixed: self.fixed || other.fixed,
        }
    }

    pub fn center_aligned(&self) -> (Self, usize, usize) {
        let (term_width, term_height) = crossterm::terminal::size().unwrap();
        let padding_x = (term_width as isize - (self.length * self.width) as isize) / 2;
        let padding_x = if padding_x <= 0 { 0 } else { padding_x } as usize;
        let space_x = " ".repeat(padding_x);

        let padding_y = (term_height as isize - self.height as isize) / 2;
        let padding_y = if padding_y <= 0 { 0 } else { padding_y } as usize;

        let mut new_data = vec![];
        for i in 0..self.height {
            let mut line = space_x.clone();
            line += &self.data[i];
            new_data.push(line);
        }
        for _ in 0..padding_y {
            let mut padding = " ".repeat(self.width * self.length);
            padding += &space_x.clone();
            new_data.push(padding);
        }

        let new_string = ArtString::from(
            new_data[0].chars().count(),
            self.height + padding_y,
            new_data,
            self.fixed,
        );

        (new_string, padding_x, padding_y)
    }
}

impl Display for ArtString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for d in &self.data {
            writeln!(f, "{}", d)?;
        }
        Ok(())
    }
}

impl std::ops::Add for ArtString {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        self.connect(other)
    }
}
