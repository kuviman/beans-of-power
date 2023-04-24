use super::*;

impl Game {
    pub fn draw_leaderboard(&self, framebuffer: &mut ugli::Framebuffer) {
        if !self.show_leaderboard {
            return;
        }
        let mut guys: Vec<&Guy> = self.guys.iter().collect();
        guys.sort_by(|a, b| match (a.progress.best_time, b.progress.best_time) {
            (Some(a), Some(b)) => a.partial_cmp(&b).unwrap(),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a
                .progress
                .best
                .partial_cmp(&b.progress.best)
                .unwrap()
                .reverse(),
        });
        let mut camera = geng::Camera2d {
            center: vec2::ZERO,
            rotation: 0.0,
            fov: 40.0,
        };
        camera.center.x += camera.fov * self.framebuffer_size.x / self.framebuffer_size.y / 2.0;
        for (place, guy) in guys.into_iter().enumerate() {
            let place = place + 1;
            let name = &guy.customization.name;
            let progress = (guy.progress.current * 100.0).round() as i32;
            let mut text = format!("#{place}: {name} - {progress}% (");
            if let Some(time) = guy.progress.best_time {
                let millis = (time * 1000.0).round() as i32;
                let seconds = millis / 1000;
                let millis = millis % 1000;
                let minutes = seconds / 60;
                let seconds = seconds % 60;
                let hours = minutes / 60;
                let minutes = minutes % 60;
                if hours != 0 {
                    text += &format!("{}:", hours);
                }
                if minutes != 0 {
                    text += &format!("{}:", minutes);
                }
                text += &format!("{}.{}", seconds, millis);
            } else {
                text += &format!("{}%", (guy.progress.best * 100.0).round() as i32);
            }
            text.push(')');
            self.geng.default_font().draw(
                framebuffer,
                &camera,
                &text,
                vec2::splat(geng::TextAlign::LEFT),
                mat3::translate(vec2(1.0, camera.fov / 2.0 - place as f32)),
                Rgba::BLACK,
            );
        }
    }
}
