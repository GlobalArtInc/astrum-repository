use qrcode::QrCode;
use qrcode::render::svg;

const MIN_SIDE: u32 = 256;
const DARK: &str = "#000000";
const LIGHT: &str = "#ffffff";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the text does not fit in a QR code")]
pub struct QrError;

pub fn svg(text: &str) -> Result<String, QrError> {
    let code = QrCode::new(text.as_bytes()).map_err(|_| QrError)?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(MIN_SIDE, MIN_SIDE)
        .quiet_zone(true)
        .dark_color(svg::Color(DARK))
        .light_color(svg::Color(LIGHT))
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_otpauth_uri_renders_as_an_svg_image() {
        let image =
            svg("otpauth://totp/Astrum:jane?secret=ABCDEFGHIJKLMNOPQRSTUVWXYZ234567&issuer=Astrum")
                .unwrap();
        assert!(image.contains("<svg"));
        assert!(image.contains("</svg>"));
    }

    #[test]
    fn text_beyond_the_symbol_capacity_is_refused() {
        assert_eq!(svg(&"x".repeat(8000)), Err(QrError));
    }
}
