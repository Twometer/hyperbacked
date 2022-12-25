use genpdf::{fonts::FontFamily, Alignment, Element, Margins};
use image::{DynamicImage, GrayImage};

use crate::{backup::BackupShare, qrcode::qrcode_encode};

pub fn print_pdf(
    share: &BackupShare,
    name: &str,
    total: usize,
) -> anyhow::Result<genpdf::Document> {
    let font = genpdf::fonts::FontData::new(include_bytes!("../res/OpenSans.ttf").to_vec(), None)?;
    let font_family = FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(),
        bold_italic: font,
    };
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("Backup");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    let mut layout = genpdf::elements::LinearLayout::vertical();

    layout.push(
        genpdf::elements::Paragraph::new(format!("{}/{}", share.number, total))
            .aligned(Alignment::Center)
            .padded(genpdf::Margins::vh(1, 0)),
    );

    let qrcode_data = qrcode_encode(&share.data)?;
    let qrcode_image = GrayImage::from_raw(1024, 1024, qrcode_data).unwrap();
    layout.push(
        genpdf::elements::Image::from_dynamic_image(DynamicImage::ImageLuma8(qrcode_image))?
            .with_alignment(Alignment::Center)
            .padded(Margins::all(1)),
    );

    layout.push(
        genpdf::elements::Paragraph::new(name)
            .aligned(Alignment::Center)
            .padded(genpdf::Margins::vh(1, 0)),
    );

    doc.push(layout.padded(Margins::all(2)).framed());
    Ok(doc)
}
