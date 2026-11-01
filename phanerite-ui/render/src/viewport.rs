/// A coherent physical render extent and its logical-to-physical scale,
/// independent of any native window type.
///
/// Layout uses fractional logical pixels. Surface dimensions are physical
/// pixels; system scaling is applied once, by the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderViewport {
    physical_size: [u32; 2],
    scale_factor: f64,
}

impl RenderViewport {
    /// Returns `None` for an empty extent or a scale not representable on the GPU.
    pub fn new(width: u32, height: u32, scale_factor: f64) -> Option<Self> {
        let gpu_scale = scale_factor as f32;
        let logical_width = (width as f64 / scale_factor) as f32;
        let logical_height = (height as f64 / scale_factor) as f32;
        if width == 0
            || height == 0
            || !gpu_scale.is_finite()
            || gpu_scale <= 0.0
            || !logical_width.is_finite()
            || !logical_height.is_finite()
            || logical_width <= 0.0
            || logical_height <= 0.0
        {
            return None;
        }
        Some(Self {
            physical_size: [width, height],
            scale_factor,
        })
    }

    #[must_use]
    pub fn physical_size(self) -> [u32; 2] {
        self.physical_size
    }

    #[must_use]
    pub fn scale_factor(self) -> f64 {
        self.scale_factor
    }

    #[must_use]
    pub fn logical_size(self) -> [f32; 2] {
        self.physical_size
            .map(|value| (value as f64 / self.scale_factor) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::RenderViewport;

    #[test]
    fn system_scale_converts_physical_extent_to_fractional_logical_pixels() {
        for (scale, expected) in [
            (1.0, [1001.0, 701.0]),
            (1.25, [800.8, 560.8]),
            (2.0, [500.5, 350.5]),
        ] {
            let viewport = RenderViewport::new(1001, 701, scale).unwrap();
            assert_eq!(viewport.logical_size(), expected);
            assert_eq!(viewport.physical_size(), [1001, 701]);
            assert_eq!(viewport.scale_factor(), scale);
        }
    }

    #[test]
    fn empty_or_unrepresentable_viewports_are_rejected() {
        assert!(RenderViewport::new(0, 100, 1.0).is_none());
        assert!(RenderViewport::new(100, 0, 1.0).is_none());
        for scale in [
            0.0,
            -1.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MIN_POSITIVE,
            f64::MAX,
        ] {
            assert!(
                RenderViewport::new(100, 100, scale).is_none(),
                "scale: {scale}"
            );
        }
    }
}
