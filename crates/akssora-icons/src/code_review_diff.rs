use crate::sink::IconSink;

// INFO: Minimal vector icon for Code Review diff header button (plus on top, minus on bottom).
pub struct CodeReviewDiffIcon;

impl CodeReviewDiffIcon {
    pub const SIZE: f32 = 14.0;

    pub fn draw<S: IconSink>(sink: &mut S, icon_x: f32, icon_y: f32, color: [f32; 4]) {
        // Top plus sign (vertical bar):
        sink.push_quad(
            icon_x + 5.5,
            icon_y + 2.0,
            icon_x + 6.8,
            icon_y + 8.5,
            color,
        );
        // Top plus sign (horizontal bar):
        sink.push_quad(
            icon_x + 3.0,
            icon_y + 4.5,
            icon_x + 9.5,
            icon_y + 5.8,
            color,
        );
        // Bottom minus bar:
        sink.push_quad(
            icon_x + 3.0,
            icon_y + 10.5,
            icon_x + 9.5,
            icon_y + 11.8,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_code_review_diff_icon_draw() {
        let mut sink = MockIconSink::default();
        CodeReviewDiffIcon::draw(&mut sink, 10.0, 10.0, [1.0; 4]);
        assert_eq!(sink.quads.len(), 3);
    }
}
