use netlify_lambda_http::{
    handler,
    lambda::{run, Context},
    IntoResponse, Request,
};
extern crate photon_rs;
use photon_rs::{base64_to_image, transform::resize, PhotonImage, Rgb};
use serde::Deserialize;
use serde_json::json;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler(respond_with_alpha)).await?;
    Ok(())
}

#[derive(Deserialize)]
struct ClientRequest {
    base64: String,
    minimum_contrast_ratio: f64,
    overlay_colour: String,
    text_colour: String,
}

/// returns contrast ratio in range 1.0 to 21.0
fn contrast_ratio_from_relative_luminance(
    relative_luminance_1: &f64,
    relative_luminance_2: &f64,
) -> f64 {
    if relative_luminance_1 < relative_luminance_2 {
        (relative_luminance_2 + 0.05) / (relative_luminance_1 + 0.05)
    } else {
        (relative_luminance_1 + 0.05) / (relative_luminance_2 + 0.05)
    }
}

/// returns contrast ratio in range 1.0 to 21.0
fn contrast_ratio(colour_1: &Rgb, colour_2: &Rgb) -> f64 {
    contrast_ratio_from_relative_luminance(
        &relative_luminance(colour_1),
        &relative_luminance(colour_2),
    )
}

// Newton-Raphson solution of delta
fn delta_from_colour_target_luminance(colour_ratio: &RgbRatio, target_luminance: f64) -> f64 {
    let mut delta_current = -0.1;
    let initial_red_ratio = colour_ratio.get_red();
    let initial_green_ratio = colour_ratio.get_green();
    let initial_blue_ratio = colour_ratio.get_blue();
    let mut colour_ratio_current = RgbRatio::new(
        initial_red_ratio + delta_current,
        initial_green_ratio + delta_current,
        initial_blue_ratio + delta_current,
    );
    let mut delta_next = delta_current
        - (relative_luminance_from_colour_ratio(&colour_ratio_current)
            / relative_luminance_derivative(&colour_ratio_current));
    let mut colour_ratio_next = RgbRatio::new(
        initial_red_ratio + delta_next,
        initial_green_ratio + delta_next,
        initial_blue_ratio + delta_next,
    );
    let mut luminance_current = relative_luminance_from_colour_ratio(&colour_ratio_next);

    let eps = 1.0e-6;
    let mut iteration = 1;

    while (target_luminance - luminance_current).abs() > eps {
        println!(
            "Iteration: {} {} {} {}",
            iteration, delta_current, delta_next, luminance_current
        );
        delta_current = delta_next;
        colour_ratio_current = colour_ratio_next;
        delta_next = delta_current
            - ((relative_luminance_from_colour_ratio(&colour_ratio_current) - target_luminance)
                / relative_luminance_derivative(&colour_ratio_current));
        colour_ratio_next = RgbRatio::new(
            initial_red_ratio + delta_next,
            initial_green_ratio + delta_next,
            initial_blue_ratio + delta_next,
        );
        luminance_current = relative_luminance_from_colour_ratio(&colour_ratio_next);
        iteration = iteration + 1;
    }
    delta_next
}

/// utility function for extract data from a data uri
fn get_data_from_data_uri(data_uri: &str) -> &str {
    let data;
    match data_uri.split(",").nth(1) {
        Some(value) => data = value,
        None => data = &data_uri,
    }
    data
}

/// convert either #000 or #000000 format colour to photon_rs::Rgb
fn get_rgb_from_hex(hex_string: &str) -> photon_rs::Rgb {
    let colour_hex = hex_string.trim();
    let hex_string_length = Some(hex_string.len());
    match hex_string_length {
        Some(7) => {
            let r = hex_to_decimal(&colour_hex[1..3]);
            let g = hex_to_decimal(&colour_hex[3..5]);
            let b = hex_to_decimal(&colour_hex[5..7]);
            Rgb::new(r, g, b)
        }
        Some(4) => {
            let r_hex = &colour_hex[1..2];
            let g_hex = &colour_hex[2..3];
            let b_hex = &colour_hex[3..4];
            let long_format_hex =
                format!("#{}{}{}{}{}{}", r_hex, r_hex, g_hex, g_hex, b_hex, b_hex);
            get_rgb_from_hex(&long_format_hex)
        }
        _ => panic!("Check rgb input"),
    }
}

/// convert an octet from hex to decimal
fn hex_to_decimal(hex_string: &str) -> u8 {
    let result = match u8::from_str_radix(&hex_string, 16) {
        Ok(num) => num,
        Err(_) => 0,
    };
    result
}

/// returns the luminance for the pixels with lowest and highest values
fn lowest_highest_luminance_rgb(image: &PhotonImage) -> (Rgb, Rgb) {
    let mut highest_luminance = 0.0;
    let mut lowest_luminance = 1.0;
    let mut highest_luminance_rgb = Rgb::new(0, 0, 0);
    let mut lowest_luminance_rgb = Rgb::new(255, 255, 255);
    let raw_pixels = image.get_raw_pixels();
    for pixel in raw_pixels.chunks(4) {
        let pixel_rgb = Rgb::new(pixel[0], pixel[1], pixel[2]);
        let pixel_luminance = relative_luminance(&pixel_rgb);
        if pixel_luminance > highest_luminance {
            highest_luminance = pixel_luminance;
            highest_luminance_rgb = pixel_rgb;
        } else if pixel_luminance < lowest_luminance {
            lowest_luminance = pixel_luminance;
            lowest_luminance_rgb = pixel_rgb;
        }
    }
    (lowest_luminance_rgb, highest_luminance_rgb)
}

/// return the minimum overlay opacity needed to produce the given contrast ratio
fn overlay_opacity(
    foreground_colour: &Rgb,
    background_colour: &Rgb,
    overlay_colour: &Rgb,
    contrast_ratio: f64,
) -> f64 {
    let foreground_luminance = relative_luminance(foreground_colour);
    let background_luminance = relative_luminance(background_colour);
    let background_colour_ratio = rgb_ratio(background_colour);
    let overlay_colour_ratio = rgb_ratio(overlay_colour);

    let composite_luminance = ((foreground_luminance + 0.05) / contrast_ratio) - 0.05;
    println!("Composite luminance: {}", composite_luminance);
    let delta = delta_from_colour_target_luminance(&background_colour_ratio, composite_luminance);

    let composite_colour_ratio = RgbRatio::new(
        background_colour_ratio.get_red() + delta,
        background_colour_ratio.get_green() + delta,
        background_colour_ratio.get_blue() + delta,
    );
    let opacity_r = (background_colour_ratio.get_red() - composite_colour_ratio.get_red())
        / (background_colour_ratio.get_red() + overlay_colour_ratio.get_red());
    let opacity_g = (background_colour_ratio.get_green() - composite_colour_ratio.get_green())
        / (background_colour_ratio.get_green() + overlay_colour_ratio.get_green());
    let opacity_b = (background_colour_ratio.get_blue() - composite_colour_ratio.get_blue())
        / (background_colour_ratio.get_blue() + overlay_colour_ratio.get_blue());
    println!("Foreground luminance: {}", foreground_luminance);
    println!("Background luminance: {}", background_luminance);
    println!("Composite luminance: {}", composite_luminance);
    println!("Delta: {}", delta);
    println!("Overlay alpha: {} {} {}", opacity_r, opacity_g, opacity_b);
    (opacity_r + opacity_g + opacity_b) / 3.0
}

/// returns the relative luminance of a colour
fn relative_luminance(colour: &Rgb) -> f64 {
    let standard_rgb_colour = rgb_ratio(colour);
    relative_luminance_from_colour_ratio(&standard_rgb_colour)
}

/// returns the first order derivative of relative lumunance wrt colours
fn relative_luminance_derivative(colour_ratio: &RgbRatio) -> f64 {
    let linear_r = if colour_ratio.get_red() <= 0.03928 {
        1.0 / 12.92
    } else {
        ((colour_ratio.get_red() + 0.055) / 1.055).powf(1.4)
    };
    let linear_g = if colour_ratio.get_green() <= 0.03928 {
        1.0 / 12.92
    } else {
        ((colour_ratio.get_green() + 0.055) / 1.055).powf(1.4)
    };
    let linear_b = if colour_ratio.get_blue() <= 0.03928 {
        1.0 / 12.92
    } else {
        ((colour_ratio.get_blue() + 0.055) / 1.055).powf(1.4)
    };
    (2.4 / 1.055) * (0.2126 * linear_r + 0.7152 * linear_g + 0.0722 * linear_b)
}

/// returns relative luminance
fn relative_luminance_from_colour_ratio(colour_ratio: &RgbRatio) -> f64 {
    let linear_r = if colour_ratio.get_red() <= 0.03928 {
        colour_ratio.get_red() / 12.92
    } else {
        ((colour_ratio.get_red() + 0.055) / 1.055).powf(2.4)
    };
    let linear_g = if colour_ratio.get_green() <= 0.03928 {
        colour_ratio.get_green() / 12.92
    } else {
        ((colour_ratio.get_green() + 0.055) / 1.055).powf(2.4)
    };
    let linear_b = if colour_ratio.get_blue() <= 0.03928 {
        colour_ratio.get_blue() / 12.92
    } else {
        ((colour_ratio.get_blue() + 0.055) / 1.055).powf(2.4)
    };
    0.2126 * linear_r + 0.7152 * linear_g + 0.0722 * linear_b
}

/// resize the image so the longest edge is 256 pixels
fn resize_image(image: &PhotonImage) -> PhotonImage {
    let long_side = 256;
    let input_height = image.get_height();
    let input_width = image.get_width();
    let is_landscape = input_height < input_width;
    let width = if is_landscape {
        long_side
    } else {
        long_side * input_width / input_height
    };
    let height = if is_landscape {
        long_side * input_height / input_width
    } else {
        long_side
    };

    resize(
        &image,
        width,
        height,
        photon_rs::transform::SamplingFilter::Lanczos3,
    )
}

/// takes input request body JSON and returns a JSON response
async fn respond_with_alpha(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let body = request.body();
    let body: ClientRequest = serde_json::from_slice(&body)?;
    let base64_image = get_data_from_data_uri(&body.base64);
    let image = base64_to_image(&base64_image);
    let resized_image = resize_image(&image);
    let lightest_rgb = lowest_highest_luminance_rgb(&resized_image).1;
    let overlay_colour = get_rgb_from_hex(&body.overlay_colour);
    let text_colour = get_rgb_from_hex(&body.text_colour);
    let alpha = overlay_opacity(
        &overlay_colour,
        &lightest_rgb,
        &text_colour,
        body.minimum_contrast_ratio,
    );
    let text_overlay_contrast = contrast_ratio(&overlay_colour, &text_colour);
    Ok(json!({ "alpha": alpha, "text_overlay_contrast": text_overlay_contrast }))
}

#[derive(Debug, PartialEq)]
struct RgbRatio {
    r: f64,
    g: f64,
    b: f64,
}

impl RgbRatio {
    fn new(r: f64, g: f64, b: f64) -> RgbRatio {
        RgbRatio { r, g, b }
    }
    fn get_red(&self) -> f64 {
        self.r
    }
    fn get_green(&self) -> f64 {
        self.g
    }
    fn get_blue(&self) -> f64 {
        self.b
    }
}

fn rgb_ratio(colour: &Rgb) -> RgbRatio {
    RgbRatio {
        r: colour.get_red() as f64 / 255.0,
        g: colour.get_green() as f64 / 255.0,
        b: colour.get_blue() as f64 / 255.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_ratio() {
        let colour_black = Rgb::new(0, 0, 0);
        let colour_blue = Rgb::new(0, 0, 255);
        let colour_white = Rgb::new(255, 255, 255);
        let colour_yellow = Rgb::new(255, 255, 0);

        assert_eq!(contrast_ratio(&colour_black, &colour_white), 21.0);
        assert_eq!(contrast_ratio(&colour_white, &colour_black), 21.0);
        assert_eq!(contrast_ratio(&colour_black, &colour_black), 1.0);
        assert_eq!(
            contrast_ratio(&colour_blue, &colour_yellow),
            8.00163666121113
        );
        assert_eq!(
            contrast_ratio(&colour_yellow, &colour_blue),
            8.00163666121113
        );
    }

    #[test]
    fn test_contrast_ratio_from_relative_luminance() {
        assert_eq!(contrast_ratio_from_relative_luminance(&0.0, &1.0), 21.0);
        assert_eq!(contrast_ratio_from_relative_luminance(&1.0, &0.0), 21.0);
        assert_eq!(contrast_ratio_from_relative_luminance(&0.0, &0.0), 1.0);
        assert_eq!(contrast_ratio_from_relative_luminance(&0.5, &0.5), 1.0);
    }

    #[test]
    fn test_delta_from_colour_target_luminance() {}

    #[test]
    fn test_get_data_from_data_uri() {
        let data_uri="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg==";
        let data="iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg==";
        assert_eq!(get_data_from_data_uri(data_uri), data);
        assert_eq!(get_data_from_data_uri(data), data);
    }

    #[test]
    fn test_get_rgb_from_hex() {
        let hex_colour = String::from("#000000");
        assert_eq!(get_rgb_from_hex(&hex_colour).get_red(), 0);
        assert_eq!(get_rgb_from_hex(&hex_colour).get_green(), 0);
        assert_eq!(get_rgb_from_hex(&hex_colour).get_blue(), 0);

        let hex_colour = String::from("#000");
        assert_eq!(get_rgb_from_hex(&hex_colour).get_red(), 0);
        assert_eq!(get_rgb_from_hex(&hex_colour).get_green(), 0);
        assert_eq!(get_rgb_from_hex(&hex_colour).get_blue(), 0);

        let hex_colour = String::from("#ff8000");
        assert_eq!(get_rgb_from_hex(&hex_colour).get_red(), 255);
        assert_eq!(get_rgb_from_hex(&hex_colour).get_green(), 128);
        assert_eq!(get_rgb_from_hex(&hex_colour).get_blue(), 0);
    }

    #[test]
    fn test_hex_to_decimal() {}

    #[test]
    fn test_lowest_highest_luminance_rgb() {}

    #[test]
    fn test_overlay_opacity() {}

    #[test]
    fn test_relative_luminance() {
        let colour_black = Rgb::new(0, 0, 0);
        let colour_blue = Rgb::new(0, 0, 255);
        let colour_white = Rgb::new(255, 255, 255);
        let colour_yellow = Rgb::new(255, 255, 0);

        assert_eq!(relative_luminance(&colour_black), 0.0);
        assert_eq!(relative_luminance(&colour_blue), 0.0722);
        assert_eq!(relative_luminance(&colour_white), 1.0);
        assert_eq!(relative_luminance(&colour_yellow), 0.9278);
    }

    #[test]
    fn test_relative_luminance_derivative() {}

    #[test]
    fn test_relative_luminance_from_colour_ratio() {}

    #[test]
    fn test_resize_image() {}

    #[test]
    fn test_respond_with_alpha() {}

    #[test]
    fn test_rgb_ratio() {
        let colour_black = Rgb::new(0, 0, 0);
        let colour_middle_grey = Rgb::new(119, 119, 119);
        let colour_orange = Rgb::new(255, 165, 0);
        let colour_white = Rgb::new(255, 255, 255);

        assert_eq!(rgb_ratio(&colour_black), (RgbRatio::new(0.0, 0.0, 0.0)));
        assert_eq!(
            rgb_ratio(&colour_middle_grey),
            (RgbRatio::new(0.4666666666666667, 0.4666666666666667, 0.4666666666666667))
        );
        assert_eq!(
            rgb_ratio(&colour_orange),
            (RgbRatio::new(1.0, 0.6470588235294118, 0.0))
        );
        assert_eq!(rgb_ratio(&colour_white), (RgbRatio::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn rgb_ratio_new() {
        let r = 0.2;
        let g = 0.4;
        let b = 0.6;
        let ratio = RgbRatio::new(r, g, b);

        assert_eq!(ratio.get_red(), 0.2);
        assert_eq!(ratio.get_green(), 0.4);
        assert_eq!(ratio.get_blue(), 0.6);
    }
}
