pub fn qrcode_encode(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let data_ascii = base85::encode(&data);
    let qrcode_data =
        qrcode_generator::to_image(&data_ascii, qrcode_generator::QrCodeEcc::Medium, 1024)?;

    Ok(qrcode_data)
}

pub fn qrcode_decode(data: &str) -> Option<Vec<u8>> {
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
