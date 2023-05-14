use super::*;

impl Game {
    pub fn update_my_guy_input(&mut self) {
        let my_guy = match self.my_guy.map(|id| self.guys.get_mut(&id).unwrap()) {
            Some(guy) => guy,
            None => return,
        };
        my_guy.paused = self.show_customizer;
        if self.show_customizer {
            return;
        }
        let mut new_input = Input {
            roll_left: 0.0,
            roll_right: 0.0,
            force_fart: false,
        };

        // Keyboard
        if CONTROLS_LEFT
            .iter()
            .any(|&key| self.geng.window().is_key_pressed(key))
        {
            new_input.roll_left = 1.0;
        }
        if CONTROLS_RIGHT
            .iter()
            .any(|&key| self.geng.window().is_key_pressed(key))
        {
            new_input.roll_right = 1.0;
        }
        if CONTROLS_FORCE_FART
            .iter()
            .any(|&key| self.geng.window().is_key_pressed(key))
            || self
                .geng
                .window()
                .is_button_pressed(geng::MouseButton::Left)
        {
            new_input.force_fart = true;
        }

        // Gamepad
        if let Some(gamepad) = self.active_gamepad {
            let gilrs = self.geng.gilrs();
            let gamepad = gilrs.gamepad(gamepad);
            for axis in [gilrs::Axis::LeftStickX, gilrs::Axis::RightStickX] {
                if let Some(axis) = gamepad.axis_data(axis) {
                    let value = axis.value();
                    if value < 0.0 {
                        new_input.roll_left += -value;
                    } else {
                        new_input.roll_right += value;
                    }
                }
            }
            for button in [gilrs::Button::South] {
                if let Some(button) = gamepad.button_data(button) {
                    if button.is_pressed() {
                        new_input.force_fart = true;
                    }
                }
            }
        }

        // Accessibility
        if let Some(radius) = self.opt.accessibility {
            let p = (self.geng.window().cursor_position().map(|x| x as f32)
                - self.framebuffer_size / 2.0)
                / radius;
            if p.x < 0.0 {
                new_input.roll_left += -p.x;
            } else {
                new_input.roll_right += p.x;
            }
            if p.y > 0.0 {
                new_input.force_fart = true;
            }
        }

        new_input.roll_left = new_input.roll_left.clamp(0.0, 1.0);
        new_input.roll_right = new_input.roll_right.clamp(0.0, 1.0);

        if my_guy.input != new_input {
            my_guy.input = new_input;
            if let Some(con) = &mut self.connection {
                con.send(ClientMessage::Update(self.simulation_time, my_guy.clone()));
            }
            if let Some(recording) = &mut self.recording {
                recording.push(self.simulation_time, my_guy);
            }
        }
    }

    pub fn update_guys(&mut self, delta_time: f32) {
        let assets = self.assets.get();
        let is_colliding = |guy: &Guy, surface_type: &str| -> bool {
            for surface in self.level.gameplay_surfaces() {
                let v = surface.vector_from(guy.state.pos);
                let penetration = guy.radius() - v.len();
                if penetration > EPS && surface.type_name == surface_type {
                    return true;
                }
            }
            false
        };
        for guy in &mut self.guys {
            if guy.paused {
                continue;
            }
            let mut time_scale = 1.0;
            for tile in self.level.gameplay_tiles() {
                if !Aabb2::points_bounding_box(tile.vertices)
                    .extend_uniform(self.config.guy_radius)
                    .contains(guy.state.pos)
                {
                    continue;
                }
                let params = &assets.tiles[&tile.type_name].params;
                if let Some(this_time_scale) = params.time_scale {
                    let percentage = circle_triangle_intersect_percentage(
                        guy.state.pos,
                        self.config.guy_radius,
                        tile.vertices,
                    );
                    time_scale *= this_time_scale.powf(percentage);
                }
            }
            let delta_time = delta_time * time_scale;

            let sfx_speed =
                (self.time_scale as f64 * time_scale as f64).powf(self.config.sfx_time_scale_power);
            if self.my_guy == Some(guy.id) {
                self.music.set_speed(sfx_speed);
            }

            let prev_state = guy.state.clone();
            let was_colliding_water = is_colliding(guy, "water");
            if (guy.state.pos - self.level.finish_point).len() < 1.5 {
                guy.progress.finished = true;
            }
            {
                let mut new_fart_type = None;
                for object in self.level.gameplay_objects() {
                    if (guy.state.pos - object.pos).len() < 1.5 {
                        if let Some(fart_type) = object.fart_type() {
                            new_fart_type = Some(fart_type.to_owned());
                        }
                    }
                }
                if let Some(new_fart_type) = new_fart_type {
                    if new_fart_type != guy.state.fart_type {
                        guy.state.fart_type = new_fart_type;
                        guy.state.fart_pressure = self.config.max_fart_pressure;
                    }
                }
            }

            // Bubble
            if let Some(time) = &mut guy.state.bubble_timer {
                *time -= delta_time;
                if *time < 0.0 {
                    guy.state.bubble_timer = None;
                }
                guy.state.vel += (guy.state.vel.normalize_or_zero()
                    * self.config.bubble_target_speed
                    - guy.state.vel)
                    .clamp_len(..=self.config.bubble_acceleration * delta_time);
            }
            for object in self.level.gameplay_objects() {
                if (guy.state.pos - object.pos).len() < 1.0 && object.type_name == "bubbler" {
                    guy.state.bubble_timer = Some(self.config.bubble_time);
                }
            }

            // This is where we do the cannon mechanics aha
            if guy.state.cannon_timer.is_none() {
                for (index, cannon) in self.level.cannons.iter().enumerate() {
                    if (guy.state.pos - cannon.pos).len() < self.config.cannon.activate_distance {
                        guy.state.long_farting = false;
                        guy.state.fart_pressure = 0.0;
                        guy.state.cannon_timer = Some(CannonTimer {
                            cannon_index: index,
                            time: self.config.cannon.shoot_time,
                        });
                    }
                }
            }
            if let Some(timer) = &mut guy.state.cannon_timer {
                let cannon = &self.level.cannons[timer.cannon_index];
                guy.state.pos = cannon.pos;
                guy.state.rot = cannon.rot - f32::PI / 2.0;
                timer.time -= delta_time;
                if timer.time < 0.0 {
                    guy.state.cannon_timer = None;
                    let dir = vec2(1.0, 0.0).rotate(cannon.rot);
                    guy.state.pos += dir * self.config.cannon.activate_distance * 1.01;
                    guy.state.vel = dir * self.config.cannon.strength;
                    guy.state.w = 0.0;

                    let mut effect = assets.cannon.shot.effect();
                    effect.set_volume(
                        (self.volume
                            * 0.6
                            * (1.0 - (guy.state.pos - self.camera.center).len() / self.camera.fov))
                            .clamp(0.0, 1.0) as f64,
                    );
                    effect.set_speed(sfx_speed);
                    effect.play();

                    let fart_type = "normal"; // TODO: not normal LUL
                    let fart_assets = &assets.farts[fart_type];
                    let farticles = self.farticles.entry(fart_type.to_owned()).or_default();
                    for _ in 0..self.config.cannon.particle_count {
                        farticles.push(Farticle {
                            size: self.config.cannon.particle_size,
                            pos: guy.state.pos,
                            vel: dir * self.config.cannon.particle_speed
                                + vec2(
                                    thread_rng().gen_range(
                                        0.0..=fart_assets.config.farticle_additional_vel,
                                    ),
                                    0.0,
                                )
                                .rotate(thread_rng().gen_range(0.0..=2.0 * f32::PI)),
                            rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
                            w: thread_rng().gen_range(
                                -fart_assets.config.farticle_w..=fart_assets.config.farticle_w,
                            ),
                            colors: self.config.cannon.particle_colors.clone(),
                            t: 1.0,
                        });
                    }
                }
                return;
            }

            if guy.progress.finished {
                guy.state.fart_pressure = 0.0;
                guy.state.rot -= delta_time;
                guy.state.pos = self.level.finish_point
                    + (guy.state.pos - self.level.finish_point)
                        .normalize_or_zero()
                        .rotate(delta_time)
                        * 1.0;
                continue;
            }

            guy.state.w += (guy.input.roll_direction().clamp(-1.0, 1.0)
                * self.config.angular_acceleration
                / guy.mass(&self.config)
                * delta_time)
                .clamp(
                    -(guy.state.w + self.config.max_angular_speed).max(0.0),
                    (self.config.max_angular_speed - guy.state.w).max(0.0),
                );

            if guy.state.bubble_timer.is_none() {
                guy.state.vel.y -= self.config.gravity * delta_time;
            }

            let mut in_water = false;
            let butt = guy.state.pos + vec2(0.0, -guy.state.radius * 0.9).rotate(guy.state.rot);
            for tile in self.level.gameplay_tiles() {
                if !Aabb2::points_bounding_box(tile.vertices)
                    .extend_uniform(self.config.guy_radius)
                    .contains(guy.state.pos)
                {
                    continue;
                }
                let percentage = circle_triangle_intersect_percentage(
                    guy.state.pos,
                    self.config.guy_radius,
                    tile.vertices,
                );
                let relative_vel = guy.state.vel - tile.flow;
                let flow_direction = tile.flow.normalize_or_zero();
                let relative_vel_along_flow = vec2::dot(flow_direction, relative_vel);
                let params = &assets.tiles[&tile.type_name].params;
                let force_along_flow =
                    -flow_direction * relative_vel_along_flow * params.friction_along_flow;
                let friction_force = -relative_vel * params.friction;
                guy.state.vel += (force_along_flow + params.additional_force + friction_force)
                    * delta_time
                    / guy.mass(&self.config)
                    * percentage;
                guy.state.w -= guy.state.w * params.friction * delta_time / guy.mass(&self.config)
                    * percentage;
                // TODO inertia?
            }
            'tile_loop: for tile in self.level.gameplay_tiles() {
                for i in 0..3 {
                    let p1 = tile.vertices[i];
                    let p2 = tile.vertices[(i + 1) % 3];
                    if vec2::skew(p2 - p1, butt - p1) < 0.0 {
                        continue 'tile_loop;
                    }
                }
                if tile.type_name == "water" {
                    in_water = true;
                }
            }

            let fart_type = if in_water {
                "bubble"
            } else {
                guy.state.fart_type.as_str()
            };
            let fart_assets = &assets.farts[fart_type];

            let could_fart = guy.state.fart_pressure >= self.config.fart_pressure_released;
            if self.config.fart_continued_force == 0.0 {
                guy.state.long_farting = false;
            }
            if guy.input.force_fart {
                if guy.state.long_farting {
                    guy.state.fart_pressure -=
                        delta_time * self.config.fart_continuation_pressure_speed;
                    if guy.state.fart_pressure < 0.0 {
                        guy.state.fart_pressure = 0.0;
                        guy.state.long_farting = false;
                    }
                } else {
                    guy.state.fart_pressure +=
                        delta_time * self.config.force_fart_pressure_multiplier;
                }
            } else {
                guy.state.long_farting = false;
                guy.state.fart_pressure += delta_time;
            };

            if !guy.state.long_farting {
                if let Some(sfx) = self.long_fart_sfx.get_mut(&guy.id) {
                    if sfx.finish_time.is_none() {
                        sfx.finish_time = Some(self.real_time);
                    }
                    let fadeout = (self.real_time - sfx.finish_time.unwrap()) / 0.2;
                    if fadeout >= 1.0 {
                        sfx.sfx.stop();
                        self.long_fart_sfx.remove(&guy.id);
                    } else {
                        sfx.sfx.set_volume(
                            (self.volume
                                * (1.0
                                    - (guy.state.pos - self.camera.center).len() / self.camera.fov))
                                .clamp(0.0, 1.0) as f64
                                * (1.0 - fadeout) as f64,
                        );
                    }
                }
            } else if let Some(sfx) = self.long_fart_sfx.get_mut(&guy.id) {
                let volume = (self.volume
                    * (1.0 - (guy.state.pos - self.camera.center).len() / self.camera.fov))
                    .clamp(0.0, 1.0) as f64;
                if fart_type != sfx.type_name {
                    // TODO: this is copypasta
                    let mut sfx = fart_assets.long_sfx.effect();
                    sfx.set_volume(volume);
                    sfx.set_speed(sfx_speed);
                    sfx.play();
                    if let Some(mut sfx) = self.long_fart_sfx.insert(
                        guy.id,
                        LongFartSfx {
                            type_name: fart_type.to_owned(),
                            finish_time: None,
                            sfx,
                        },
                    ) {
                        sfx.sfx.stop();
                    }
                } else {
                    sfx.sfx.set_volume(volume);
                    sfx.sfx.set_speed(sfx_speed);
                }
            } else {
                log::warn!("No sfx for long fart?");
            }

            if guy.state.long_farting {
                guy.animation.next_farticle_time -= delta_time;
                while guy.animation.next_farticle_time < 0.0 {
                    guy.animation.next_farticle_time +=
                        1.0 / fart_assets.config.long_fart_farticles_per_second;
                    self.farticles
                        .entry(fart_type.to_owned())
                        .or_default()
                        .push(Farticle {
                            size: 1.0,
                            pos: butt,
                            vel: guy.state.vel
                                + vec2(
                                    thread_rng().gen_range(
                                        0.0..=fart_assets.config.farticle_additional_vel,
                                    ),
                                    0.0,
                                )
                                .rotate(thread_rng().gen_range(0.0..=2.0 * f32::PI))
                                + vec2(0.0, -fart_assets.config.long_fart_farticle_speed)
                                    .rotate(guy.state.rot),
                            rot: if fart_assets.config.farticle_random_rotation {
                                thread_rng().gen_range(0.0..2.0 * f32::PI)
                            } else {
                                0.0
                            },
                            w: thread_rng().gen_range(
                                -fart_assets.config.farticle_w..=fart_assets.config.farticle_w,
                            ),
                            colors: fart_assets.config.colors.get(),
                            t: 1.0,
                        });
                }
                guy.state.vel += vec2(0.0, self.config.fart_continued_force * delta_time)
                    .rotate(guy.state.rot)
                    / guy.mass(&self.config);
            } else if (guy.state.fart_pressure >= self.config.fart_pressure_released
                && guy.input.force_fart)
                || guy.state.fart_pressure >= self.config.max_fart_pressure
            {
                guy.state.bubble_timer = None;
                guy.state.fart_pressure -= self.config.fart_pressure_released;
                guy.state.long_farting = true;
                {
                    let mut sfx = fart_assets.long_sfx.effect();
                    sfx.set_volume(0.0);
                    sfx.set_speed(sfx_speed);
                    sfx.play();
                    if let Some(mut sfx) = self.long_fart_sfx.insert(
                        guy.id,
                        LongFartSfx {
                            type_name: fart_type.to_owned(),
                            finish_time: None,
                            sfx,
                        },
                    ) {
                        sfx.sfx.stop();
                    }
                }
                let farticles = self.farticles.entry(fart_type.to_owned()).or_default();
                for _ in 0..fart_assets.config.farticle_count {
                    farticles.push(Farticle {
                        size: 1.0,
                        pos: butt,
                        vel: guy.state.vel
                            + vec2(
                                thread_rng()
                                    .gen_range(0.0..=fart_assets.config.farticle_additional_vel),
                                0.0,
                            )
                            .rotate(thread_rng().gen_range(0.0..=2.0 * f32::PI)),
                        rot: if fart_assets.config.farticle_random_rotation {
                            thread_rng().gen_range(0.0..2.0 * f32::PI)
                        } else {
                            0.0
                        },
                        w: thread_rng().gen_range(
                            -fart_assets.config.farticle_w..=fart_assets.config.farticle_w,
                        ),
                        colors: fart_assets.config.colors.get(),
                        t: 1.0,
                    });
                }
                guy.state.vel += vec2(0.0, self.config.fart_strength).rotate(guy.state.rot)
                    / guy.mass(&self.config);
                let mut effect = fart_assets.sfx.choose(&mut thread_rng()).unwrap().effect();
                effect.set_volume(
                    (self.volume
                        * (1.0 - (guy.state.pos - self.camera.center).len() / self.camera.fov))
                        .clamp(0.0, 1.0) as f64,
                );
                effect.set_speed(sfx_speed);
                effect.play();
            } else if !could_fart && guy.state.fart_pressure >= self.config.fart_pressure_released {
                // Growling stomach recharge
                if Some(guy.id) == self.my_guy {
                    let mut effect = assets.sfx.fart_recharge.effect();
                    effect.set_volume(self.volume as f64 * 0.5);
                    effect.play();
                }
                guy.animation.growl_progress = Some(0.0);
            }

            if let Some(growl) = &mut guy.animation.growl_progress {
                *growl += delta_time / self.config.growl_time;
                if *growl >= 1.0 {
                    guy.animation.growl_progress = None;
                }
            }

            guy.state.vel += guy.state.stick_force / guy.mass(&self.config) * delta_time;
            guy.state.stick_force -= guy
                .state
                .stick_force
                .clamp_len(..=self.config.stick_force_fadeout_speed * delta_time);

            guy.state.pos += guy.state.vel * delta_time;
            guy.state.rot += guy.state.w * delta_time;

            struct Collision<'a> {
                penetration: f32,
                normal: vec2<f32>,
                surface: &'a Surface,
                assets: &'a SurfaceAssets,
            }

            let mut collision_to_resolve = None;
            let mut was_colliding_water = was_colliding_water;
            for surface in self.level.gameplay_surfaces() {
                let from_surface = -surface.vector_from(guy.state.pos);
                let penetration = guy.radius() - from_surface.len();
                if penetration > 0.0 {
                    let surface_assets = &assets.surfaces[&surface.type_name];

                    if surface.type_name == "water" && !was_colliding_water {
                        was_colliding_water = true;
                        if vec2::dot(from_surface, guy.state.vel).abs() > 0.5 {
                            let mut effect = assets.sfx.water_splash.effect();
                            effect.set_volume(
                                (self.volume
                                    * 0.6
                                    * (1.0
                                        - (guy.state.pos - self.camera.center).len()
                                            / self.camera.fov))
                                    .clamp(0.0, 1.0) as f64,
                            );
                            effect.set_speed(sfx_speed);
                            effect.play();
                            let fart_type = "bubble";
                            let fart_assets = &assets.farts[fart_type];
                            let farticles = self.farticles.entry(fart_type.to_owned()).or_default();
                            for _ in 0..30 {
                                farticles.push(Farticle {
                                    size: 0.6,
                                    pos: guy.state.pos - from_surface
                                        + vec2(
                                            thread_rng().gen_range(-guy.radius()..=guy.radius()),
                                            0.0,
                                        ),
                                    vel: {
                                        let mut v = vec2(0.0, thread_rng().gen_range(0.0..=1.0))
                                            .rotate(
                                                thread_rng()
                                                    .gen_range(-f32::PI / 4.0..=f32::PI / 4.0),
                                            );
                                        v.y *= 0.3;
                                        v * 2.0
                                    },
                                    rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
                                    w: thread_rng().gen_range(
                                        -fart_assets.config.farticle_w
                                            ..=fart_assets.config.farticle_w,
                                    ),
                                    colors: fart_assets.config.colors.get(),
                                    t: 0.5,
                                });
                            }
                        }
                    }

                    if surface_assets.params.non_collidable {
                        continue;
                    }
                    let normal = from_surface.normalize_or_zero();
                    let normal_vel = vec2::dot(normal, guy.state.vel);
                    if normal_vel < -EPS
                        && normal_vel > -surface_assets.params.fallthrough_speed.unwrap_or(1e9)
                        && vec2::skew(surface.p2 - surface.p1, normal) > 0.0
                        && penetration < self.config.max_penetration
                    {
                        let collision = Collision {
                            penetration,
                            surface,
                            normal,
                            assets: surface_assets,
                        };
                        collision_to_resolve = std::cmp::max_by_key(
                            collision_to_resolve,
                            Some(collision),
                            |collision| match collision {
                                Some(collision) => (
                                    r32(collision.penetration),
                                    r32(vec2::skew(
                                        (collision.surface.p2 - collision.surface.p1)
                                            .normalize_or_zero(),
                                        collision.normal,
                                    )),
                                ),
                                None => (r32(-1.0), r32(0.0)),
                            },
                        );
                    }
                }
            }

            if let Some(collision) = collision_to_resolve {
                guy.state.bubble_timer = None;

                let before = guy.state.clone();

                let normal_vel = vec2::dot(guy.state.vel, collision.normal);
                let tangent = collision.normal.rotate_90();
                let tangent_vel = vec2::dot(guy.state.vel, tangent) - guy.state.w * guy.radius()
                    + collision.surface.flow;
                let bounce_impulse = -normal_vel * (1.0 + collision.assets.params.bounciness);
                let impulse =
                    bounce_impulse.max(-normal_vel + collision.assets.params.min_bounce_vel);
                guy.state.vel += collision.normal * impulse / guy.mass(&self.config);
                let max_friction_impulse = normal_vel.abs() * collision.assets.params.friction;
                let friction_impulse = -tangent_vel.clamp_abs(max_friction_impulse);

                guy.state.pos += collision.normal * collision.penetration;
                guy.state.vel += tangent * friction_impulse / guy.mass(&self.config);
                guy.state.w -= friction_impulse / guy.radius() / guy.mass(&self.config);

                guy.state.vel -=
                    guy.state.vel * (delta_time * collision.assets.params.speed_friction).min(1.0);
                guy.state.w -=
                    guy.state.w * (delta_time * collision.assets.params.rotation_friction).min(1.0);

                // Stickiness
                guy.state.stick_force = std::cmp::max_by_key(
                    guy.state.stick_force,
                    (normal_vel * collision.assets.params.stick_strength)
                        .clamp_abs(collision.assets.params.max_stick_force)
                        * collision.normal,
                    |force| r32(force.len()),
                );

                // Snow layer
                if collision.surface.type_name == "snow" {
                    guy.state.snow_layer += guy.state.w.abs() * delta_time * 1e-2;
                }

                {
                    let snow_falloff = ((bounce_impulse.abs()
                        - self.config.snow_falloff_impulse_min)
                        / (self.config.snow_falloff_impulse_max
                            - self.config.snow_falloff_impulse_min))
                        .clamp(0.0, 1.0)
                        * self.config.max_snow_layer
                        * collision.assets.params.snow_falloff;
                    let snow_falloff = snow_falloff.min(guy.state.snow_layer);
                    guy.state.snow_layer -= snow_falloff;
                    let fart_type = "normal"; // TODO: not normal?
                    let fart_assets = &assets.farts[fart_type];
                    let farticles = self.farticles.entry(fart_type.to_owned()).or_default();
                    for _ in 0..(100.0 * snow_falloff / self.config.max_snow_layer) as i32 {
                        farticles.push(Farticle {
                            size: 0.6,
                            pos: guy.state.pos
                                + vec2(guy.radius(), 0.0)
                                    .rotate(thread_rng().gen_range(0.0..2.0 * f32::PI)),
                            vel: thread_rng().gen_circle(before.vel, 1.0),
                            rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
                            w: thread_rng().gen_range(
                                -fart_assets.config.farticle_w..=fart_assets.config.farticle_w,
                            ),
                            colors: self.config.snow_particle_colors.clone(),
                            t: 0.5,
                        });
                    }
                }
                guy.state.snow_layer = guy.state.snow_layer.clamp(0.0, self.config.max_snow_layer);

                if let Some(sound) = &collision.assets.sound {
                    let volume = ((-0.5 + impulse / 2.0) / 2.0).clamp(0.0, 1.0);
                    if volume > 0.0 {
                        let mut effect = sound.effect();
                        effect.set_volume(
                            (self.volume
                                * volume
                                * (1.0
                                    - (guy.state.pos - self.camera.center).len() / self.camera.fov))
                                .clamp(0.0, 1.0) as f64,
                        );
                        effect.set_speed(sfx_speed);
                        effect.play();
                    }
                }
            } else {
                guy.state.stick_force = vec2::ZERO;
            }

            // Portals
            for portal in &self.level.portals {
                let is_colliding =
                    |pos: vec2<f32>| -> bool { (pos - portal.pos).len() < self.config.portal.size };
                if !is_colliding(prev_state.pos) && is_colliding(guy.state.pos) {
                    if let Some(dest) = portal.dest {
                        guy.state.pos = self.level.portals[dest].pos;
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_connection(&mut self) {
        let messages: Vec<ServerMessage> = match &mut self.connection {
            Some(con) => con.new_messages().collect::<anyhow::Result<_>>().unwrap(),
            None => return,
        };
        for message in messages {
            match message {
                ServerMessage::ForceReset => {
                    self.respawn_my_guy();
                }
                ServerMessage::Pong => {
                    if let Some(con) = &mut self.connection {
                        con.send(ClientMessage::Ping);
                        if let Some(id) = self.my_guy {
                            let guy = self.guys.get(&id).unwrap();
                            con.send(ClientMessage::Update(self.simulation_time, guy.clone()));
                        }
                    }
                }
                ServerMessage::ClientId(_) => unreachable!(),
                ServerMessage::UpdateGuy(t, guy) => {
                    match self.remote_updates.entry(guy.id) {
                        std::collections::hash_map::Entry::Occupied(mut e) => {
                            e.get_mut().push(t, &guy);
                        }
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(Replay::new(t, &guy));
                        }
                    }
                    if let Some(current) = self.guys.get_mut(&guy.id) {
                        current.progress = guy.progress;
                    }
                }
                ServerMessage::Despawn(id) => {
                    self.guys.remove(&id);
                    self.remote_updates.remove(&id);
                }
                ServerMessage::Emote(id, emote) => {
                    self.emotes.retain(|&(_, x, _)| x != id);
                    self.emotes.push((self.real_time, id, emote));
                }
            }
        }
    }

    pub fn respawn_my_guy(&mut self) {
        // COPYPASTA MMMMM 🍝 or is it anymore?
        let new_guy = Guy::new(self.client_id, self.level.spawn_point, true, &self.config);
        if self.my_guy.is_none() {
            self.my_guy = Some(self.client_id);
        }
        self.guys.insert(new_guy);
        self.simulation_time = 0.0;
        if let Some(con) = &mut self.connection {
            con.send(ClientMessage::Despawn);
        }
    }
}
