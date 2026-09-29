use crate::color::Color;
use nalgebra_glm::Vec3;
use std::f32::consts::PI;
use std::fs;

/// Panorama equirectangular HDR (Kiara 1 Dawn, Poly Haven) decodificado a
/// color plano una sola vez al cargar, para poder samplearlo sin bloquear
/// los hilos de render. El raylib que trae este proyecto se compila sin
/// soporte para .hdr, así que parseamos el formato Radiance nosotros mismos.
pub struct Skybox {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl Skybox {
    pub fn load(path: &str) -> Self {
        let bytes = fs::read(path).expect("no se pudo leer el skybox");
        let (width, height, rgbe) = decode_radiance_hdr(&bytes);

        let pixels = rgbe
            .chunks_exact(4)
            .map(|p| rgbe_to_color(p[0], p[1], p[2], p[3]))
            .collect();

        Skybox { width, height, pixels }
    }

    /// Proyección equirectangular: dirección del rayo -> color de fondo.
    pub fn sample(&self, direction: &Vec3) -> Color {
        let d = direction.normalize();

        let u = 0.5 + d.z.atan2(d.x) / (2.0 * PI);
        let v = 0.5 - d.y.clamp(-1.0, 1.0).asin() / PI;

        let x = ((u * self.width as f32) as usize).min(self.width - 1);
        let y = ((v * self.height as f32) as usize).min(self.height - 1);

        self.pixels[y * self.width + x]
    }
}

/// Los valores de un HDR pueden superar 1.0 (sol, cielo brillante); clampeamos
/// en vez de dejar que se desborden al convertir a 8 bits.
fn rgbe_to_color(r: u8, g: u8, b: u8, e: u8) -> Color {
    if e == 0 {
        return Color::new(0, 0, 0);
    }

    let scale = 2f32.powi(e as i32 - 128 - 8);
    let to_u8 = |channel: u8| ((channel as f32 * scale).clamp(0.0, 1.0) * 255.0) as u8;

    Color::new(to_u8(r), to_u8(g), to_u8(b))
}

/// Decodifica un archivo Radiance .hdr (el formato que exporta Poly Haven)
/// a una lista plana de píxeles RGBE (4 bytes por píxel, fila por fila).
fn decode_radiance_hdr(bytes: &[u8]) -> (usize, usize, Vec<u8>) {
    let mut pos = 0;

    // Header ASCII: líneas "CLAVE=valor" hasta una línea en blanco.
    loop {
        let line_end = bytes[pos..].iter().position(|&b| b == b'\n').unwrap() + pos;
        let empty = line_end == pos;
        pos = line_end + 1;
        if empty {
            break;
        }
    }

    // Línea de resolución, ej. "-Y 1024 +X 2048" (alto y ancho, en cualquier orden).
    let res_end = bytes[pos..].iter().position(|&b| b == b'\n').unwrap() + pos;
    let res_line = std::str::from_utf8(&bytes[pos..res_end]).unwrap();
    pos = res_end + 1;

    let tokens: Vec<&str> = res_line.split_whitespace().collect();
    let mut width = 0usize;
    let mut height = 0usize;
    for pair in tokens.chunks(2) {
        if let [axis, value] = pair {
            let n: usize = value.parse().unwrap();
            if axis.ends_with('X') {
                width = n;
            } else if axis.ends_with('Y') {
                height = n;
            }
        }
    }

    let mut pixels = vec![0u8; width * height * 4];

    for y in 0..height {
        let row = &mut pixels[y * width * 4..(y + 1) * width * 4];

        // Scanline RLE "nuevo estilo": empieza con 2 2 <ancho en 16 bits>.
        let is_new_rle = width >= 8
            && width < 0x8000
            && bytes[pos] == 2
            && bytes[pos + 1] == 2
            && ((bytes[pos + 2] as usize) << 8 | bytes[pos + 3] as usize) == width;

        if is_new_rle {
            pos += 4;
            for channel in 0..4 {
                let mut x = 0;
                while x < width {
                    let count = bytes[pos];
                    pos += 1;

                    if count > 128 {
                        let run = (count - 128) as usize;
                        let value = bytes[pos];
                        pos += 1;
                        for _ in 0..run {
                            row[x * 4 + channel] = value;
                            x += 1;
                        }
                    } else {
                        let run = count as usize;
                        for i in 0..run {
                            row[(x + i) * 4 + channel] = bytes[pos + i];
                        }
                        pos += run;
                        x += run;
                    }
                }
            }
        } else {
            // Scanline plano (sin comprimir): width píxeles RGBE seguidos.
            for x in 0..width {
                row[x * 4..x * 4 + 4].copy_from_slice(&bytes[pos..pos + 4]);
                pos += 4;
            }
        }
    }

    (width, height, pixels)
}
