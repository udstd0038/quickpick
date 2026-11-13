//! Explicit Windows-only synthetic clipboard roundtrip. Never captures the screen.
#[cfg(windows)]
fn main() {
    if !std::env::args().any(|arg| arg == "--replace-clipboard-with-synthetic-image") {
        eprintln!(
            "Requires --replace-clipboard-with-synthetic-image; this replaces the clipboard with test pixels."
        );
        std::process::exit(2);
    }
    if let Err(error) = verify() {
        eprintln!("Synthetic clipboard roundtrip failed: {error}");
        std::process::exit(1);
    }
    println!(
        "Synthetic CF_DIB and CF_BITMAP roundtrips passed (9 copies). No screen capture or image file was created."
    );
}

#[cfg(windows)]
fn verify() -> Result<(), &'static str> {
    for _ in 0..3 {
        for (width, height) in [(32, 24), (5, 3), (1, 2)] {
            let source = image::RgbaImage::from_fn(width, height, |x, y| {
                image::Rgba([(x * 7) as u8, (y * 11) as u8, ((x + y) * 3) as u8, 255])
            });
            quickpick_platform::probe::copy_image(&source)?;
            let _clipboard = clipboard_win::Clipboard::new_attempts(10)
                .map_err(|_| "could not reopen clipboard")?;
            if !clipboard_win::raw::is_format_avail(clipboard_win::formats::CF_DIB)
                || !clipboard_win::raw::is_format_avail(clipboard_win::formats::CF_BITMAP)
            {
                return Err("missing bitmap format");
            }
            let mut dib = Vec::new();
            clipboard_win::raw::get_vec(clipboard_win::formats::CF_DIB, &mut dib)
                .map_err(|_| "could not read CF_DIB")?;
            let decoder =
                image::codecs::bmp::BmpDecoder::new_without_file_header(std::io::Cursor::new(&dib))
                    .map_err(|_| "could not decode CF_DIB")?;
            let dib_image = image::DynamicImage::from_decoder(decoder)
                .map_err(|_| "could not decode CF_DIB pixels")?
                .to_rgb8();
            let mut bitmap = Vec::new();
            clipboard_win::raw::get_bitmap(&mut bitmap).map_err(|_| "could not read CF_BITMAP")?;
            let bitmap_image =
                image::load_from_memory_with_format(&bitmap, image::ImageFormat::Bmp)
                    .map_err(|_| "could not decode CF_BITMAP")?
                    .to_rgb8();
            for actual in [&dib_image, &bitmap_image] {
                if actual.dimensions() != source.dimensions()
                    || source
                        .pixels()
                        .zip(actual.pixels())
                        .any(|(expected, pixel)| expected.0[..3] != pixel.0)
                {
                    return Err("bitmap dimensions or RGB pixels differ from synthetic source");
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("The clipboard roundtrip requires Windows; no clipboard was accessed.");
    std::process::exit(2);
}
