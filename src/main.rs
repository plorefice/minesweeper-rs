use failure::{format_err, Error};
use piston_window as pw;

use std::path::Path;

mod game;
use game::*;

fn main() -> Result<(), Error> {
    let field_size = (10, 10);

    let mut window: pw::PistonWindow = pw::WindowSettings::new(
        "Minesweeper",
        (CELL_SIZE.0 * field_size.0, CELL_SIZE.1 * field_size.1),
    )
    .opengl(pw::OpenGL::V4_1)
    .exit_on_esc(true)
    .build()
    .unwrap();

    let mut field = Field::new(
        &mut rand::thread_rng(),
        field_size,
        12,
        decode_tileset("res/tileset.jpg", &mut window.factory)?,
    );

    while let Some(e) = window.next() {
        use pw::{MouseCursorEvent, PressEvent};

        window.draw_2d(&e, |c, g| {
            pw::clear([1.0; 4], g);
            field.render(c, g);
        });

        if let Some(p) = e.mouse_cursor_args() {
            field.mouse_move(&p);
        }

        if let Some(e) = e.press_args() {
            field.mouse_click(&e);
        }
    }
    Ok(())
}

fn decode_tileset<P: AsRef<Path>>(
    p: P,
    f: &mut pw::GfxFactory,
) -> Result<Vec<pw::G2dTexture>, Error> {
    use image::{jpeg::JPEGDecoder, ImageDecoder};
    use std::fs::File;

    let decoder = JPEGDecoder::new(File::open(p)?)?;

    let (w, h) = {
        let (w, h) = decoder.dimensions();
        (w as u32, h as u32)
    };
    let (nx, ny) = (w / TILE_SIZE.0, h / TILE_SIZE.1);

    let row_len = decoder.row_bytes() as u32;
    let pixel_size = row_len / w;

    if decoder.colortype() != image::ColorType::RGB(8) {
        return Err(format_err!(
            "color type not supported: {:?}",
            decoder.colortype()
        ));
    }

    let data = decoder.read_image()?;

    let mut imbuf = image::RgbaImage::new(TILE_SIZE.0, TILE_SIZE.1);
    let mut texvec = Vec::with_capacity((nx * ny) as usize);

    for j in 0..ny {
        for i in 0..nx {
            let tl = (i * TILE_SIZE.0 * pixel_size, j * TILE_SIZE.1 * row_len);

            for y in 0..TILE_SIZE.1 {
                for x in 0..TILE_SIZE.0 {
                    let py = (tl.1 + y * row_len) as usize;
                    let px = (tl.0 + x * pixel_size) as usize;

                    imbuf.put_pixel(
                        x,
                        y,
                        image::Rgba([
                            data[py + px],
                            data[py + px + 1],
                            data[py + px + 2],
                            std::u8::MAX,
                        ]),
                    );
                }
            }

            texvec.push(pw::Texture::from_image(
                f,
                &imbuf,
                &pw::TextureSettings::new().filter(pw::Filter::Nearest),
            )?)
        }
    }

    Ok(texvec)
}
