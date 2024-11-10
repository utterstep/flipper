use std::path::Path;

use color_eyre::eyre::{Result, WrapErr};
use plotters::{
    prelude::*,
    style::colors::full_palette::{BLUEGREY_900, GREEN_900, RED_900},
};

use crate::dump::SavedSignal;

fn round_to(x: u32, round_to: u32) -> u32 {
    (x + round_to / 2) / round_to * round_to
}

const IMAGE_WIDTH: u32 = 1512 * 2;
const IMAGE_HEIGHT: u32 = 800 * 2;

/// Plots the signal.
///
/// Signal is an IR signal that has been saved to a file.
/// The signal data is a series of timings in microseconds.
/// The first value is the duration of the first pulse,
/// the second value is the duration of the pause after that,
/// the third value is the duration of the second pulse, and so on.
///
/// The signal is plotted as a series of rectangles.
/// Each rectangle represents a pulse or a pause.
/// The width of the rectangle is the duration of the pulse or pause.
///
/// The height of the rectangle is 200 px in the case of a pulse,
/// and 20 px in the case of a pause.
///
/// The rectangles are colored green for pulses and red for pauses.
pub fn plot_signal(signal: &SavedSignal, out_folder: &Path) -> Result<()> {
    let out_path = out_folder.join(format!("{}.png", signal.name()));
    let root = BitMapBackend::new(&out_path, (IMAGE_WIDTH, IMAGE_HEIGHT)).into_drawing_area();
    // use dark theme
    root.fill(&BLUEGREY_900)
        .wrap_err("Failed to fill background")?;

    let positive_signal_style = ShapeStyle {
        color: GREEN_900.mix(0.8),
        filled: true,
        stroke_width: 1,
    };

    let negative_signal_style = ShapeStyle {
        color: RED_900.mix(0.8),
        filled: true,
        stroke_width: 1,
    };

    const ROUND_TO: u32 = 550;
    let rounded_signal = signal
        .data()
        .iter()
        .map(|x| round_to(*x, ROUND_TO))
        .collect::<Vec<u32>>();

    let total_timing: u32 = rounded_signal.iter().sum();
    let x_limit = total_timing.max(round_to(300_000, ROUND_TO));

    // use white sans-serif font for the captions
    let font = ("sans-serif", 20).into_font().color(&WHITE);

    let mut chart = ChartBuilder::on(&root)
        .caption(signal.name(), font)
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..x_limit, 0..300)
        .wrap_err("Failed to build chart")?;

    chart
        .configure_mesh()
        .draw()
        .wrap_err("Failed to draw mesh")?;

    // The signal is plotted as a series of rectangles.
    // Each rectangle represents a pulse or a pause.
    // The width of the rectangle is the duration of the pulse or pause.
    //
    // The height of the rectangle is 200 px in the case of a pulse,
    // and 20 px in the case of a pause.
    //
    // The rectangles are colored green for pulses and red for pauses.

    let mut x = 0;

    chart
        .draw_series(rounded_signal.iter().enumerate().map(|(i, timing)| {
            let x0 = x;
            x += timing;
            let x1 = x;
            let y0 = 0;
            let y1 = if i & 1 == 0 { 200 } else { 20 };

            let style = if i & 1 == 0 {
                positive_signal_style.clone()
            } else {
                negative_signal_style.clone()
            };

            Rectangle::new([(x0, y0), (x1, y1)], style)
        }))
        .wrap_err("Failed to draw series")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_to() {
        assert_eq!(round_to(0, 550), 0);
        assert_eq!(round_to(1, 550), 0);
        assert_eq!(round_to(549, 550), 550);
        assert_eq!(round_to(550, 550), 550);
        assert_eq!(round_to(551, 550), 550);
        assert_eq!(round_to(120, 50), 100);
        assert_eq!(round_to(125, 50), 150);
    }
}
