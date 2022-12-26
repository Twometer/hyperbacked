use image::ImageBuffer;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};

pub fn qrcode_encode(data: &[u8], size: usize) -> anyhow::Result<Vec<u8>> {
    let data_ascii = base85::encode(&data);
    let qrcode_data =
        qrcode_generator::to_image(&data_ascii, qrcode_generator::QrCodeEcc::Medium, size)?;

    Ok(qrcode_data)
}

pub fn qrcode_scan() -> anyhow::Result<Vec<u8>> {
    let mut camera = Camera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution),
    )?;
    camera.open_stream()?;

    let decoder = bardecoder::default_decoder();

    loop {
        let frame = camera.frame()?;
        let frame_img = frame.decode_image::<RgbFormat>()?;

        let raw_img = frame_img.as_raw();

        let barcodes = decoder.decode(&image::DynamicImage::ImageRgb8(
            ImageBuffer::from_raw(frame_img.width(), frame_img.height(), raw_img.to_owned())
                .expect("Freshly exported image must be valid"),
        ));

        let first_barcode = barcodes
            .iter()
            .map_while(|barcode| barcode.as_ref().ok())
            .nth(0);

        if let Some(barcode_str) = first_barcode {
            if let Some(barcode_data) = qrcode_decode(&barcode_str) {
                return Ok(barcode_data);
            }
        }
    }
}

fn qrcode_decode(data: &str) -> Option<Vec<u8>> {
    if !is_valid_base85(data) {
        return None;
    }
    base85::decode(data)
}

fn is_valid_base85(data: &str) -> bool {
    data.chars().all(|c| match c {
        '0'..='9' => true,
        'A'..='Z' => true,
        'a'..='z' => true,
        '!' => true,
        '#' => true,
        '$' => true,
        '%' => true,
        '&' => true,
        '(' => true,
        ')' => true,
        '*' => true,
        '+' => true,
        '-' => true,
        ';' => true,
        '<' => true,
        '=' => true,
        '>' => true,
        '?' => true,
        '@' => true,
        '^' => true,
        '_' => true,
        '`' => true,
        '{' => true,
        '|' => true,
        '}' => true,
        '~' => true,
        _ => false,
    })
}
