use genpdf::{fonts::FontFamily, Alignment, Element, Margins};
use image::{DynamicImage, GrayImage};

use crate::{backup::BackupShard, qrcode::qrcode_encode};

const QRCODE_SIZE: u32 = 1024;

pub fn print_pdf(
    shard: &BackupShard,
    backup_name: &str,
    total_shards: usize,
) -> anyhow::Result<genpdf::Document> {
    let font = genpdf::fonts::FontData::new(include_bytes!("../res/OpenSans.ttf").to_vec(), None)?;
    let font_family = FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(),
        bold_italic: font,
    };
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(backup_name);

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    let mut layout = genpdf::elements::LinearLayout::vertical();

    layout.push(
        genpdf::elements::Paragraph::new(format!("{}/{}", shard.number, total_shards))
            .aligned(Alignment::Center)
            .padded(genpdf::Margins::vh(1, 0)),
    );

    let qrcode_data = qrcode_encode(&shard.data, QRCODE_SIZE as usize)?;
    let qrcode_image = GrayImage::from_raw(QRCODE_SIZE, QRCODE_SIZE, qrcode_data)
        .expect("QR Code generator created invalid image");
    layout.push(
        genpdf::elements::Image::from_dynamic_image(DynamicImage::ImageLuma8(qrcode_image))?
            .with_alignment(Alignment::Center)
            .padded(Margins::all(1)),
    );

    layout.push(
        genpdf::elements::Paragraph::new(backup_name)
            .aligned(Alignment::Center)
            .padded(genpdf::Margins::vh(1, 0)),
    );

    doc.push(layout.padded(Margins::all(2)).framed());
    Ok(doc)
}
