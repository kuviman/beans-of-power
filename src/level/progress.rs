use super::*;

impl Level {
    pub fn progress_at(&self, pos: vec2<f32>) -> f32 {
        let mut total_len = 0.0;
        for window in self.expected_path.windows(2) {
            let a = window[0];
            let b = window[1];
            total_len += (b - a).len();
        }
        let mut progress = 0.0;
        let mut closest_point_distance = 1e9;
        let mut prefix_len = 0.0;
        for window in self.expected_path.windows(2) {
            let a = window[0];
            let b = window[1];
            let v = Surface {
                p1: a,
                p2: b,
                type_name: String::new(),
            }
            .vector_from(pos);
            if v.len() < closest_point_distance {
                closest_point_distance = v.len();
                progress = (prefix_len + (pos + v - a).len()) / total_len;
            }
            prefix_len += (b - a).len();
        }
        progress
    }
}
