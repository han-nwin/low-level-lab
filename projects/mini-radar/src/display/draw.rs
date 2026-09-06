use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Arc, Circle, Line, PrimitiveStyle, Rectangle},
};
use tinytga::Tga;

use crate::sensor::TargetInfo;

const RADAR_ORIGIN: Point = Point::new(120, 120);
const RADAR_RADIUS_PX: i32 = 115;
const MAX_RANGE_MM: i32 = 6000;

const RADAR_LEFT_EDGE: Point = Point::new(20, 178);
const RADAR_RIGHT_EDGE: Point = Point::new(220, 178);

pub const SWEEP_STEP_COUNT: usize = 13;

// Endpoints on the 6 m ring, from -60 degrees to +60 degrees.
const SWEEP_ENDPOINTS: [Point; SWEEP_STEP_COUNT] = [
    Point::new(220, 178),
    Point::new(208, 194),
    Point::new(194, 208),
    Point::new(178, 220),
    Point::new(159, 228),
    Point::new(140, 233),
    Point::new(120, 235),
    Point::new(100, 233),
    Point::new(81, 228),
    Point::new(62, 220),
    Point::new(46, 208),
    Point::new(32, 194),
    Point::new(20, 178),
];

pub fn draw_screen<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    display.clear(Rgb565::BLACK)?;
    draw_radar_background(display)
}

pub fn draw_radar_background<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let boundary_style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 2);
    let grid_style = PrimitiveStyle::with_stroke(Rgb565::new(0, 18, 0), 1);

    // Full-screen range rings form the persistent radar face.
    for diameter in [46, 92, 138, 184, 230] {
        Circle::with_center(RADAR_ORIGIN, diameter)
            .into_styled(grid_style)
            .draw(display)?;
    }

    // Full circular radial grid, like a traditional radar scope.
    for (start, end) in [
        (Point::new(5, 120), Point::new(235, 120)),
        (Point::new(9, 90), Point::new(231, 150)),
        (Point::new(20, 62), Point::new(220, 178)),
        (Point::new(39, 39), Point::new(201, 201)),
        (Point::new(62, 20), Point::new(178, 220)),
        (Point::new(90, 9), Point::new(150, 231)),
        (Point::new(120, 5), Point::new(120, 235)),
    ] {
        Line::new(start, end)
            .into_styled(grid_style)
            .draw(display)?;
    }

    // Bright lines and arc show the LD2450's actual +/-60 degree scan area.
    Line::new(RADAR_ORIGIN, RADAR_LEFT_EDGE)
        .into_styled(boundary_style)
        .draw(display)?;
    Line::new(RADAR_ORIGIN, RADAR_RIGHT_EDGE)
        .into_styled(boundary_style)
        .draw(display)?;

    Arc::with_center(RADAR_ORIGIN, 230, 30.0.deg(), 120.0.deg())
        .into_styled(boundary_style)
        .draw(display)?;

    Circle::with_center(RADAR_ORIGIN, 7)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(display)
}

pub fn draw_sweep<D>(display: &mut D, step: usize, color: Rgb565) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    Line::new(RADAR_ORIGIN, SWEEP_ENDPOINTS[step])
        .into_styled(PrimitiveStyle::with_stroke(color, 2))
        .draw(display)
}

pub fn erase_targets<D>(display: &mut D, targets: &[TargetInfo; 3]) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    for target in targets {
        if target.x_coordinate == 0 && target.y_coordinate == 0 {
            continue;
        }

        Rectangle::new(
            target_to_point(target) - Point::new(10, 10),
            Size::new(21, 21),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(display)?;
    }

    Ok(())
}

pub fn draw_targets<D>(display: &mut D, targets: &[TargetInfo; 3]) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    for target in targets {
        // Ignore an unused target slot.
        if target.x_coordinate == 0 && target.y_coordinate == 0 {
            continue;
        }

        let point = target_to_point(target);

        draw_target_marker(display, point)?;
    }

    Ok(())
}

fn draw_target_marker<D>(display: &mut D, center: Point) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    const HALF_SIZE: i32 = 9;
    const CORNER_LENGTH: i32 = 7;

    let left = center.x - HALF_SIZE;
    let right = center.x + HALF_SIZE;
    let top = center.y - HALF_SIZE;
    let bottom = center.y + HALF_SIZE;
    let style = PrimitiveStyle::with_stroke(Rgb565::RED, 2);

    // Four broken corners create the lock-on [*] shape.
    for (start, end) in [
        (Point::new(left, top), Point::new(left + CORNER_LENGTH, top)),
        (Point::new(left, top), Point::new(left, top + CORNER_LENGTH)),
        (
            Point::new(right - CORNER_LENGTH, top),
            Point::new(right, top),
        ),
        (
            Point::new(right, top),
            Point::new(right, top + CORNER_LENGTH),
        ),
        (
            Point::new(left, bottom),
            Point::new(left + CORNER_LENGTH, bottom),
        ),
        (
            Point::new(left, bottom - CORNER_LENGTH),
            Point::new(left, bottom),
        ),
        (
            Point::new(right - CORNER_LENGTH, bottom),
            Point::new(right, bottom),
        ),
        (
            Point::new(right, bottom - CORNER_LENGTH),
            Point::new(right, bottom),
        ),
    ] {
        Line::new(start, end).into_styled(style).draw(display)?;
    }

    Circle::with_center(center, 7)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(display)
}

fn target_to_point(target: &TargetInfo) -> Point {
    let x = i32::from(target.x_coordinate);
    let y = i32::from(target.y_coordinate);

    Point::new(
        RADAR_ORIGIN.x - x * RADAR_RADIUS_PX / MAX_RANGE_MM,
        RADAR_ORIGIN.y + y * RADAR_RADIUS_PX / MAX_RANGE_MM,
    )
}

pub fn draw_rust_logo<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let tga: Tga<Rgb565> =
        Tga::from_slice(include_bytes!("../../assets/rust-logo-180.tga")).unwrap(); // 24 bits
    // image

    // For a 128x128 image:
    // (240 - 128) / 2 = 56
    let image = Image::new(&tga, Point::new(56, 56));

    image.draw(display)
}
