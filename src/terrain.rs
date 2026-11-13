//! Shaded relief and water layers for the scenario's topographic map.
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const WIDTH: u32 = 1120;
const HEIGHT: u32 = 720;

pub fn relief_image() -> Image {
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let p = Vec2::new(
                (x as f32 / WIDTH as f32 - 0.5) * 2.0,
                (0.5 - y as f32 / HEIGHT as f32) * 2.0,
            );
            let z = elevation(p);
            let slope = (z - elevation(p + Vec2::new(-0.009, 0.012))) * 5.0;
            let river_center = 0.22 * (p.y * 3.2).sin() + p.y * 0.23;
            let bank = (p.x - river_center).abs();
            let grain =
                ((x.wrapping_mul(1_664_525) ^ y.wrapping_mul(1_013_904_223)) % 31) as f32 / 1000.0;
            let ground = Vec3::new(0.18, 0.23, 0.18)
                + Vec3::new(0.16, 0.14, 0.10) * z
                + Vec3::splat(slope + grain);
            let contour = 1.0 - ((z * 18.0).fract() - 0.5).abs() * 2.0;
            let shaded = ground - Vec3::splat(0.047 * contour.powi(9));
            let rgb = if bank < 0.021 {
                Vec3::new(0.11, 0.22, 0.23) + Vec3::splat(grain)
            } else if bank < 0.028 {
                shaded.lerp(Vec3::new(0.35, 0.37, 0.27), 0.28)
            } else {
                shaded
            };
            pixels.extend(
                rgb.clamp(Vec3::ZERO, Vec3::ONE)
                    .to_array()
                    .map(|v| (v * 255.0) as u8),
            );
            pixels.push(255);
        }
    }
    Image::new(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn elevation(p: Vec2) -> f32 {
    let ridge = |center: Vec2, scale: Vec2| {
        let q = (p - center) * scale;
        (-q.length_squared()).exp()
    };
    0.12 + 0.52 * ridge(Vec2::new(-0.55, 0.45), Vec2::new(2.3, 3.6))
        + 0.62 * ridge(Vec2::new(0.54, -0.30), Vec2::new(2.9, 2.9))
        + 0.42 * ridge(Vec2::new(0.56, 0.67), Vec2::new(2.8, 4.1))
        + 0.38 * ridge(Vec2::new(-0.72, -0.57), Vec2::new(3.6, 3.8))
        + 0.042 * (p.x * 13.0 + p.y * 3.0).sin() * (p.y * 14.0).cos()
}
