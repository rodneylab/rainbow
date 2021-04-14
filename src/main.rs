extern crate photon_rs;
use photon_rs::native::open_image;
use photon_rs::transform::resize;
use photon_rs::PhotonImage;
use photon_rs::Rgb;

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

    let linear_r = if standard_rgb_colour.get_red() <= 0.03928 {
        standard_rgb_colour.get_red() / 12.92
    } else {
        ((standard_rgb_colour.get_red() + 0.055) / 1.055).powf(2.4)
    };
    let linear_g = if standard_rgb_colour.get_green() <= 0.03928 {
        standard_rgb_colour.get_green() / 12.92
    } else {
        ((standard_rgb_colour.get_green() + 0.055) / 1.055).powf(2.4)
    };
    let linear_b = if standard_rgb_colour.get_blue() <= 0.03928 {
        standard_rgb_colour.get_blue() / 12.92
    } else {
        ((standard_rgb_colour.get_blue() + 0.055) / 1.055).powf(2.4)
    };
    0.2126 * linear_r + 0.7152 * linear_g + 0.0722 * linear_b
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
