use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use image::GenericImageView;
use rayon::prelude::*;

static HEX_LUT: [[u8; 2]; 256] = {
    const H: &[u8] = b"0123456789ABCDEF";
    let mut t = [[0u8; 2]; 256];
    let mut i = 0usize;
    while i < 256 {
        t[i][0] = H[i >> 4];
        t[i][1] = H[i & 0xF];
        i += 1;
    }
    t
};

#[inline(always)]
fn write_dec(buf: &mut Vec<u8>, v: usize, width: usize) {
    let start = buf.len();
    buf.resize(start + width, b'0');
    let mut n = v;
    for i in (0..width).rev() {
        buf[start + i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
}

#[inline(always)]
fn write_hex(buf: &mut Vec<u8>, v: u8) {
    let h = HEX_LUT[v as usize];
    buf.push(h[0]);
    buf.push(h[1]);
}

fn digits(n: usize) -> usize {
    if n == 0 { return 1; }
    let mut d = 0;
    let mut v = n;
    while v > 0 { v /= 10; d += 1; }
    d
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: volt <path_to_image>");
        return;
    }

    let raw_arg = args[1].trim_matches('"').to_string();

    // Resolve relative paths against the current working directory
    let img_path: PathBuf = {
        let p = PathBuf::from(&raw_arg);
        if p.is_absolute() {
            p
        } else {
            env::current_dir().expect("Cannot read current directory").join(p)
        }
    };

    let img = match image::open(&img_path) {
        Ok(i) => i,
        Err(e) => {
            println!("Error: Could not open image: {}", e);
            println!("Looked for file at: {}", img_path.display());
            return;
        }
    };

    let (width, height) = img.dimensions();
    let w = width as usize;
    let h = height as usize;

    let x_digits = digits(w.saturating_sub(1));
    let y_digits = digits(h.saturating_sub(1));

    let rgba = img.to_rgba8();
    let raw = rgba.as_raw();

    // Write the output next to the image, not relative to cwd
    let txt_path = img_path.with_extension("txt");
    let txt_name = txt_path.to_str().unwrap();

    println!("Processing {}...", img_path.display());

    let rows: Vec<Vec<u8>> = (0..h)
        .into_par_iter()
        .map(|y| {
            let line_len = x_digits + y_digits + 9;
            let mut row_buf = Vec::with_capacity(w * line_len);

            let row_offset = y * w * 4;
            for x in 0..w {
                let idx = row_offset + x * 4;
                let r = raw[idx];
                let g = raw[idx + 1];
                let b = raw[idx + 2];
                let a = raw[idx + 3];

                if a == 0 { continue; }

                write_dec(&mut row_buf, x, x_digits);
                row_buf.push(b' ');
                write_dec(&mut row_buf, y, y_digits);
                row_buf.extend_from_slice(b" #");
                write_hex(&mut row_buf, r);
                write_hex(&mut row_buf, g);
                write_hex(&mut row_buf, b);
                row_buf.push(b'\n');
            }
            row_buf
        })
        .collect();

    let txt_file = File::create(txt_name).expect("Failed to create text file");
    let mut writer = BufWriter::with_capacity(1024 * 1024, txt_file);
    for row in rows {
        writer.write_all(&row).unwrap();
    }

    println!("Finished! Created: {}", txt_name);
}
