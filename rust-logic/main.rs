use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use image::{GenericImageView, RgbaImage};

const MAGIC: &[u8]    = b"volt";
const VERSION: u8     = 0x01;
const OP_FILL_BG: u8  = 0x01;
const OP_RECT: u8     = 0x02;
const OP_RASTER: u8   = 0x06;
const OP_EOF: u8      = 0xFF;
const FLAG_ALPHA: u8  = 0x01;
const FLAG_PALETTE: u8 = 0x02;
const PAL_GLOBAL: u8  = 0x00;
const PAL_RGB: u8     = 0x03;
const PAL_RGBA: u8    = 0x04;
const MIN_RECT_AREA: usize = 14;

#[inline(always)]
fn get_px(raw: &[u8], x: usize, y: usize, w: usize) -> [u8; 4] {
    let i = (y * w + x) * 4;
    [raw[i], raw[i + 1], raw[i + 2], raw[i + 3]]
}

fn find_background(raw: &[u8]) -> [u8; 4] {
    let mut counts: HashMap<[u8; 4], u32> = HashMap::new();
    for chunk in raw.chunks_exact(4) {
        *counts.entry([chunk[0], chunk[1], chunk[2], chunk[3]]).or_insert(0) += 1;
    }
    counts.into_iter().max_by_key(|(_, v)| *v).map(|(k, _)| k).unwrap()
}

fn extract_palette(raw: &[u8]) -> Option<Vec<[u8; 4]>> {
    let mut seen: HashSet<[u8; 4]> = HashSet::new();
    for chunk in raw.chunks_exact(4) {
        seen.insert([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if seen.len() > 255 {
            return None;
        }
    }
    Some(seen.into_iter().collect())
}

fn detect_rectangles(
    raw: &[u8],
    covered: &mut Vec<bool>,
    w: usize,
    h: usize,
) -> Vec<(u16, u16, u16, u16, [u8; 4])> {
    let mut rects = Vec::new();

    for y in 0..h {
        for x in 0..w {
            if covered[y * w + x] {
                continue;
            }
            let color = get_px(raw, x, y, w);

            let mut rw = 0;
            while x + rw < w && !covered[y * w + x + rw] && get_px(raw, x + rw, y, w) == color {
                rw += 1;
            }

            let mut rh = 1;
            let mut cur_w = rw;
            while y + rh < h {
                let mut row_w = 0;
                while row_w < cur_w {
                    let nx = x + row_w;
                    if covered[(y + rh) * w + nx] || get_px(raw, nx, y + rh, w) != color {
                        break;
                    }
                    row_w += 1;
                }
                if row_w == 0 {
                    break;
                }
                cur_w = cur_w.min(row_w);
                rh += 1;
            }
            rw = cur_w;

            if rw * rh < MIN_RECT_AREA {
                continue;
            }

            for dy in 0..rh {
                for dx in 0..rw {
                    covered[(y + dy) * w + x + dx] = true;
                }
            }
            rects.push((x as u16, y as u16, rw as u16, rh as u16, color));
        }
    }
    rects
}

fn push_color(
    out: &mut Vec<u8>,
    color: [u8; 4],
    has_alpha: bool,
    pal_map: &Option<HashMap<[u8; 4], u8>>,
) {
    if let Some(pm) = pal_map {
        out.push(*pm.get(&color).unwrap_or(&0));
    } else if has_alpha {
        out.extend_from_slice(&color);
    } else {
        out.extend_from_slice(&color[..3]);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: volt <path_to_image>");
        std::process::exit(1);
    }

    let raw_arg = args[1].trim_matches('"').to_string();
    let img_path: PathBuf = {
        let p = PathBuf::from(&raw_arg);
        if p.is_absolute() {
            p
        } else {
            env::current_dir().expect("Cannot read cwd").join(p)
        }
    };

    let img: RgbaImage = match image::open(&img_path) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Looked for: {}", img_path.display());
            std::process::exit(1);
        }
    };

    let (width, height) = img.dimensions();
    let w = width as usize;
    let h = height as usize;
    let raw = img.as_raw();

    println!("Encoding {}x{} image...", w, h);

    let has_alpha = raw.chunks_exact(4).any(|px| px[3] < 255);
    let palette    = extract_palette(raw);
    let bg_color   = find_background(raw);

    let bg_count: usize = raw.chunks_exact(4)
        .filter(|px| [px[0], px[1], px[2], px[3]] == bg_color)
        .count();
    let use_bg_fill = bg_count > w * h / 4;

    let mut covered = vec![false; w * h];
    if use_bg_fill {
        for y in 0..h {
            for x in 0..w {
                if get_px(raw, x, y, w) == bg_color {
                    covered[y * w + x] = true;
                }
            }
        }
    }

    let rects = detect_rectangles(raw, &mut covered, w, h);
    println!("Found {} rectangles", rects.len());

    let pal_map: Option<HashMap<[u8; 4], u8>> = palette.as_ref().map(|pal| {
        pal.iter().enumerate().map(|(i, &c)| (c, i as u8)).collect()
    });

    let mut out: Vec<u8> = Vec::new();

    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&(w as u16).to_le_bytes());
    out.extend_from_slice(&(h as u16).to_le_bytes());

    let mut flags: u8 = 0;
    if has_alpha          { flags |= FLAG_ALPHA; }
    if palette.is_some()  { flags |= FLAG_PALETTE; }
    out.push(flags);

    if let Some(ref pal) = palette {
        out.push(pal.len() as u8);
        for &color in pal {
            if has_alpha { out.extend_from_slice(&color); }
            else         { out.extend_from_slice(&color[..3]); }
        }
    }

    if use_bg_fill {
        out.push(OP_FILL_BG);
        push_color(&mut out, bg_color, has_alpha, &pal_map);
    }

    for &(rx, ry, rw, rh, color) in &rects {
        out.push(OP_RECT);
        out.extend_from_slice(&rx.to_le_bytes());
        out.extend_from_slice(&ry.to_le_bytes());
        out.extend_from_slice(&rw.to_le_bytes());
        out.extend_from_slice(&rh.to_le_bytes());
        push_color(&mut out, color, has_alpha, &pal_map);
    }

    let pal_type = if pal_map.is_some() { PAL_GLOBAL }
                   else if has_alpha     { PAL_RGBA }
                   else                  { PAL_RGB };

    for y in 0..h {
        let mut x = 0;
        while x < w {
            if covered[y * w + x] { x += 1; continue; }

            let start_x = x;
            while x < w && !covered[y * w + x] { x += 1; }
            let block_w = x - start_x;

            let mut rle: Vec<u8> = Vec::new();
            let mut px = start_x;
            while px < x {
                let color = get_px(raw, px, y, w);
                let mut count: usize = 1;
                while (px + count) < x && count < 255
                    && get_px(raw, px + count, y, w) == color
                {
                    count += 1;
                }
                rle.push(count as u8);
                if let Some(ref pm) = pal_map {
                    rle.push(*pm.get(&color).unwrap_or(&0));
                } else if has_alpha {
                    rle.extend_from_slice(&color);
                } else {
                    rle.extend_from_slice(&color[..3]);
                }
                px += count as usize;
            }

            out.push(OP_RASTER);
            out.extend_from_slice(&(start_x as u16).to_le_bytes());
            out.extend_from_slice(&(y as u16).to_le_bytes());
            out.extend_from_slice(&(block_w as u16).to_le_bytes());
            out.extend_from_slice(&1u16.to_le_bytes());
            out.push(pal_type);
            out.extend_from_slice(&(rle.len() as u32).to_le_bytes());
            out.extend_from_slice(&rle);
        }
    }

    out.push(OP_EOF);

    let out_path = img_path.with_extension("volt");
    let out_name = out_path.to_str().expect("Invalid output path");
    let file = File::create(out_name).expect("Failed to create output file");
    let mut writer = BufWriter::new(file);
    writer.write_all(&out).expect("Failed to write output");

    let orig_size = std::fs::metadata(&img_path).map(|m| m.len()).unwrap_or(0);
    let volt_size = out.len() as u64;
    println!(
        "Done! {} bytes ({:.1}% of original {}B) -> {}",
        volt_size,
        if orig_size > 0 { volt_size as f64 / orig_size as f64 * 100.0 } else { 0.0 },
        orig_size,
        out_name
    );
}
