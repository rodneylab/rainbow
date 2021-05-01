extern crate photon_rs;
use photon_rs::native::open_image;
use photon_rs::transform::resize;
use photon_rs::PhotonImage;
use photon_rs::Rgb;
use std::io;

fn main() {
    let img = open_image("image.jpg");
    let resized_image = resize_image(&img);

    let min_max_luminance = lowest_highest_luminance(&resized_image);
    println!("Lowest luminance: {}", min_max_luminance.0);
    println!("Highest luminance: {}", min_max_luminance.1);

    let white_contrast_ratio = contrast_ratio_from_relative_luminance(
        &relative_luminance(&Rgb::new(255, 255, 255)),
        &min_max_luminance.1,
    );
    let black_contrast_ratio = contrast_ratio_from_relative_luminance(
        &relative_luminance(&Rgb::new(0, 0, 0)),
        &min_max_luminance.0,
    );
    println!("White luminance: {}", white_contrast_ratio);
    println!("Black luminance: {}", black_contrast_ratio);

    let mixed = composite_colour(&Rgb::new(0, 0, 0), &Rgb::new(255, 255, 255), &1.0);
    let contrast = contrast_ratio(&mixed, &Rgb::new(255, 255, 255));
    println!("Overlay contrast ratio: {}", contrast);

    // read_in_colour();

    let lightest_rgb = lowest_highest_luminance_rgb(&resized_image).1;
    println!("lightest background: {:?}", lightest_rgb);
    let alpha = overlay_opacity(
        &Rgb::new(255, 255, 255),
        &lightest_rgb,
        &Rgb::new(0, 0, 0),
        4.5,
    );
    println!("Overlay alpha: {}", alpha);

    // check
    let new_lightest = composite_colour(&lightest_rgb, &Rgb::new(0, 0, 0), &alpha);
    println!("new lightest: {:?}", new_lightest);
    let actual_contrast_ratio = contrast_ratio(&new_lightest, &Rgb::new(255, 255, 255));
    println!("Actual contrast ratio: {}", actual_contrast_ratio);
}

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

fn composite_colour(base_colour: &Rgb, overlay_colour: &Rgb, overlay_opacity: &f64) -> Rgb {
    let r = base_colour.get_red() as f64 * (1.0 - overlay_opacity)
        + overlay_colour.get_red() as f64 * overlay_opacity;
    let g = base_colour.get_green() as f64 * (1.0 - overlay_opacity)
        + overlay_colour.get_green() as f64 * overlay_opacity;
    let b = base_colour.get_blue() as f64 * (1.0 - overlay_opacity)
        + overlay_colour.get_blue() as f64 * overlay_opacity;
    Rgb::new(r.trunc() as u8, g.trunc() as u8, b.trunc() as u8)
    // Rgb::new(r.ceil() as u8, g.ceil() as u8, b.ceil() as u8)
}

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
        // while iteration < 11 {
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

fn lowest_highest_luminance(image: &PhotonImage) -> (f64, f64) {
    let mut highest_luminance = 0.0;
    let mut lowest_luminance = 1.0;
    let raw_pixels = image.get_raw_pixels();
    for pixel in raw_pixels.chunks(4) {
        let pixel_rgb = Rgb::new(pixel[0], pixel[1], pixel[2]);
        let pixel_luminance = relative_luminance(&pixel_rgb);
        if pixel_luminance > highest_luminance {
            highest_luminance = pixel_luminance;
        } else if pixel_luminance < lowest_luminance {
            lowest_luminance = pixel_luminance;
        }
    }
    (lowest_luminance, highest_luminance)
}

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

fn read_in_colour() -> photon_rs::Rgb {
    println!("Hex colour: (e.g. \"#ff0044\")");
    let mut colour_hex = String::new();
    io::stdin()
        .read_line(&mut colour_hex)
        .expect("Sorry, I don't understand, try somthing like '#ff0044'");
    let colour_hex = colour_hex.trim();
    let r = match u8::from_str_radix(&colour_hex[1..3], 16) {
        Ok(num) => num,
        Err(_) => 0,
    };
    let g = match u8::from_str_radix(&colour_hex[3..5], 16) {
        Ok(num) => num,
        Err(_) => 0,
    };
    let b = match u8::from_str_radix(&colour_hex[5..7], 16) {
        Ok(num) => num,
        Err(_) => 0,
    };
    println!("{} {} {}", r, g, b);
    Rgb::new(r, g, b)
}

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

fn relative_luminance(colour: &Rgb) -> f64 {
    let standard_rgb_colour = rgb_ratio(colour);
    relative_luminance_from_colour_ratio(&standard_rgb_colour)
}

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
    fn test_lowest_highest_luminance() {
        let input_image = open_image("image.jpg");
        let result = lowest_highest_luminance(&input_image);

        assert!(result.0 >= 0.0);
        assert!(result.0 <= 1.0);
        assert!(result.1 >= 0.0);
        assert!(result.1 <= 1.0);
        assert!(result.0 <= result.1);
    }

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
    fn test_resize_image() {
        let input_image = open_image("image.jpg");
        let output_image = resize_image(&input_image);
        assert_eq!(output_image.get_width(), 170);
        assert_eq!(output_image.get_height(), 256);

        let input_image = open_image("image-1.jpg");
        let output_image = resize_image(&input_image);
        assert_eq!(output_image.get_width(), 256);
        assert_eq!(output_image.get_height(), 192);
    }

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
