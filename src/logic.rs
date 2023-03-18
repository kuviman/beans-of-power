use super::*;

impl Game {
    pub fn update_my_guy_input(&mut self) {
        if self.show_customizer {
            return;
        }
        let my_guy = match self.my_guy.map(|id| self.guys.get_mut(&id).unwrap()) {
            Some(guy) => guy,
            None => return,
        };
        let new_input = Input {
            roll_left: if CONTROLS_LEFT
                .iter()
                .any(|&key| self.geng.window().is_key_pressed(key))
            {
                1.0
            } else {
                0.0
            },
            roll_right: if CONTROLS_RIGHT
                .iter()
                .any(|&key| self.geng.window().is_key_pressed(key))
            {
                1.0
            } else {
                0.0
            },
            force_fart: CONTROLS_FORCE_FART
                .iter()
                .any(|&key| self.geng.window().is_key_pressed(key)),
        };
        if my_guy.input != new_input {
            my_guy.input = new_input;
            if let Some(con) = &mut self.connection {
                con.send(ClientMessage::Update(self.simulation_time, my_guy.clone()));
            }
        }
    }

    pub fn update_guys(&mut self, delta_time: f32) {
        let assets = self.assets.get();
        let is_colliding = |guy: &Guy, surface_type: &str| -> bool {
            for surface in self.level.gameplay_surfaces() {
                let v = surface.vector_from(guy.ball.pos);
                let penetration = guy.radius() - v.len();
                if penetration > EPS && surface.type_name == surface_type {
                    return true;
                }
            }
            false
        };
        for guy in &mut self.guys {
            let mut time_scale = 1.0;
            for tile in self.level.gameplay_tiles() {
                if !Aabb2::points_bounding_box(tile.vertices)
                    .extend_uniform(self.config.guy_radius)
                    .contains(guy.ball.pos)
                {
                    continue;
                }
                let params = &assets.tiles[&tile.type_name].params;
                if let Some(this_time_scale) = params.time_scale {
                    let percentage = circle_triangle_intersect_percentage(
                        guy.ball.pos,
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

            let prev_state = guy.ball.clone();
            let was_colliding_water = is_colliding(guy, "water");
            if (guy.ball.pos - self.level.finish_point).len() < 1.5 {
                guy.progress.finished = true;
            }
            {
                let mut new_fart_type = None;
                for object in self.level.gameplay_objects() {
                    if (guy.ball.pos - object.pos).len() < 1.5 {
                        if let Some(fart_type) = object.fart_type() {
                            new_fart_type = Some(fart_type.to_owned());
                        }
                    }
                }
                if let Some(new_fart_type) = new_fart_type {
                    if new_fart_type != guy.fart_type {
                        guy.fart_type = new_fart_type;
                        guy.fart_state.fart_pressure = self.config.max_fart_pressure;
                    }
                }
            }

            // Bubble
            if let Some(time) = &mut guy.bubble_timer {
                *time -= delta_time;
                if *time < 0.0 {
                    guy.bubble_timer = None;
                }
                guy.ball.vel += (guy.ball.vel.normalize_or_zero()
                    * self.config.bubble_target_speed
                    - guy.ball.vel)
                    .clamp_len(..=self.config.bubble_acceleration * delta_time);
            }
            for object in self.level.gameplay_objects() {
                if (guy.ball.pos - object.pos).len() < 1.0 && object.type_name == "bubbler" {
                    guy.bubble_timer = Some(self.config.bubble_time);
                }
            }

            // This is where we do the cannon mechanics aha
            if guy.cannon_timer.is_none() {
                for (index, cannon) in self.level.cannons.iter().enumerate() {
                    if (guy.ball.pos - cannon.pos).len() < self.config.cannon.activate_distance {
                        guy.fart_state = default();
                        guy.cannon_timer = Some(CannonTimer {
                            cannon_index: index,
                            time: self.config.cannon.shoot_time,
                        });
                    }
                }
            }
            if let Some(timer) = &mut guy.cannon_timer {
                let cannon = &self.level.cannons[timer.cannon_index];
                guy.ball.pos = cannon.pos;
                guy.ball.rot = cannon.rot - f32::PI / 2.0;
                timer.time -= delta_time;
                if timer.time < 0.0 {
                    guy.cannon_timer = None;
                    let dir = vec2(1.0, 0.0).rotate(cannon.rot);
                    guy.ball.pos += dir * self.config.cannon.activate_distance * 1.01;
                    guy.ball.vel = dir * self.config.cannon.strength;
                    guy.ball.w = 0.0;

                    let mut effect = assets.cannon.shot.effect();
                    effect.set_volume(
                        (self.volume
                            * 0.6
                            * (1.0 - (guy.ball.pos - self.camera.center).len() / self.camera.fov))
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
                            pos: guy.ball.pos,
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
                guy.fart_state.fart_pressure = 0.0;
                guy.ball.rot -= delta_time;
                guy.ball.pos = self.level.finish_point
                    + (guy.ball.pos - self.level.finish_point)
                        .normalize_or_zero()
                        .rotate(delta_time)
                        * 1.0;
                continue;
            }

            guy.ball.w += (guy.input.roll_direction().clamp(-1.0, 1.0)
                * self.config.angular_acceleration
                / guy.mass(&self.config)
                * delta_time)
                .clamp(
                    -(guy.ball.w + self.config.max_angular_speed).max(0.0),
                    (self.config.max_angular_speed - guy.ball.w).max(0.0),
                );

            if guy.bubble_timer.is_none() {
                guy.ball.vel.y -= self.config.gravity * delta_time;
            }

            let mut in_water = false;
            let butt = guy.ball.pos + vec2(0.0, -guy.ball.radius * 0.9).rotate(guy.ball.rot);
            for tile in self.level.gameplay_tiles() {
                if !Aabb2::points_bounding_box(tile.vertices)
                    .extend_uniform(self.config.guy_radius)
                    .contains(guy.ball.pos)
                {
                    continue;
                }
                let percentage = circle_triangle_intersect_percentage(
                    guy.ball.pos,
                    self.config.guy_radius,
                    tile.vertices,
                );
                let relative_vel = guy.ball.vel - tile.flow;
                let flow_direction = tile.flow.normalize_or_zero();
                let relative_vel_along_flow = vec2::dot(flow_direction, relative_vel);
                let params = &assets.tiles[&tile.type_name].params;
                let force_along_flow =
                    -flow_direction * relative_vel_along_flow * params.friction_along_flow;
                let friction_force = -relative_vel * params.friction;
                guy.ball.vel += (force_along_flow + params.additional_force + friction_force)
                    * delta_time
                    / guy.mass(&self.config)
                    * percentage;
                guy.ball.w -=
                    guy.ball.w * params.friction * delta_time / guy.mass(&self.config) * percentage;
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
                guy.fart_type.as_str()
            };
            let fart_assets = &assets.farts[fart_type];

            let could_fart = guy.fart_state.fart_pressure >= self.config.fart_pressure_released;
            if self.config.fart_continued_force == 0.0 {
                guy.fart_state.long_farting = false;
            }
            if guy.input.force_fart {
                if guy.fart_state.long_farting {
                    guy.fart_state.fart_pressure -=
                        delta_time * self.config.fart_continuation_pressure_speed;
                    if guy.fart_state.fart_pressure < 0.0 {
                        guy.fart_state.fart_pressure = 0.0;
                        guy.fart_state.long_farting = false;
                    }
                } else {
                    guy.fart_state.fart_pressure +=
                        delta_time * self.config.force_fart_pressure_multiplier;
                }
            } else {
                guy.fart_state.long_farting = false;
                guy.fart_state.fart_pressure += delta_time;
            };

            if !guy.fart_state.long_farting {
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
                                    - (guy.ball.pos - self.camera.center).len() / self.camera.fov))
                                .clamp(0.0, 1.0) as f64
                                * (1.0 - fadeout) as f64,
                        );
                    }
                }
            } else if let Some(sfx) = self.long_fart_sfx.get_mut(&guy.id) {
                let volume = (self.volume
                    * (1.0 - (guy.ball.pos - self.camera.center).len() / self.camera.fov))
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
                warn!("No sfx for long fart?");
            }

            if guy.fart_state.long_farting {
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
                            vel: guy.ball.vel
                                + vec2(
                                    thread_rng().gen_range(
                                        0.0..=fart_assets.config.farticle_additional_vel,
                                    ),
                                    0.0,
                                )
                                .rotate(thread_rng().gen_range(0.0..=2.0 * f32::PI))
                                + vec2(0.0, -fart_assets.config.long_fart_farticle_speed)
                                    .rotate(guy.ball.rot),
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
                guy.ball.vel += vec2(0.0, self.config.fart_continued_force * delta_time)
                    .rotate(guy.ball.rot)
                    / guy.mass(&self.config);
            } else if (guy.fart_state.fart_pressure >= self.config.fart_pressure_released
                && guy.input.force_fart)
                || guy.fart_state.fart_pressure >= self.config.max_fart_pressure
            {
                guy.bubble_timer = None;
                guy.fart_state.fart_pressure -= self.config.fart_pressure_released;
                guy.fart_state.long_farting = true;
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
                        vel: guy.ball.vel
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
                guy.ball.vel += vec2(0.0, self.config.fart_strength).rotate(guy.ball.rot)
                    / guy.mass(&self.config);
                let mut effect = fart_assets.sfx.choose(&mut thread_rng()).unwrap().effect();
                effect.set_volume(
                    (self.volume
                        * (1.0 - (guy.ball.pos - self.camera.center).len() / self.camera.fov))
                        .clamp(0.0, 1.0) as f64,
                );
                effect.set_speed(sfx_speed);
                effect.play();
            } else if !could_fart
                && guy.fart_state.fart_pressure >= self.config.fart_pressure_released
            {
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

            guy.ball.vel += guy.stick_force / guy.mass(&self.config) * delta_time;
            guy.stick_force -= guy
                .stick_force
                .clamp_len(..=self.config.stick_force_fadeout_speed * delta_time);

            guy.ball.pos += guy.ball.vel * delta_time;
            guy.ball.rot += guy.ball.w * delta_time;

            struct Collision<'a> {
                penetration: f32,
                normal: vec2<f32>,
                surface: &'a Surface,
                assets: &'a SurfaceAssets,
            }

            let mut collision_to_resolve = None;
            let mut was_colliding_water = was_colliding_water;
            for surface in self.level.gameplay_surfaces() {
                let from_surface = -surface.vector_from(guy.ball.pos);
                let penetration = guy.radius() - from_surface.len();
                if penetration > 0.0 {
                    let surface_assets = &assets.surfaces[&surface.type_name];

                    if surface.type_name == "water" && !was_colliding_water {
                        was_colliding_water = true;
                        if vec2::dot(from_surface, guy.ball.vel).abs() > 0.5 {
                            let mut effect = assets.sfx.water_splash.effect();
                            effect.set_volume(
                                (self.volume
                                    * 0.6
                                    * (1.0
                                        - (guy.ball.pos - self.camera.center).len()
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
                                    pos: guy.ball.pos - from_surface
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
                    let normal_vel = vec2::dot(normal, guy.ball.vel);
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
                guy.bubble_timer = None;

                let before = guy.ball.clone();

                let normal_vel = vec2::dot(guy.ball.vel, collision.normal);
                let tangent = collision.normal.rotate_90();
                let tangent_vel = vec2::dot(guy.ball.vel, tangent) - guy.ball.w * guy.radius()
                    + collision.surface.flow;
                let bounce_impulse = -normal_vel * (1.0 + collision.assets.params.bounciness);
                let impulse =
                    bounce_impulse.max(-normal_vel + collision.assets.params.min_bounce_vel);
                guy.ball.vel += collision.normal * impulse / guy.mass(&self.config);
                let max_friction_impulse = normal_vel.abs() * collision.assets.params.friction;
                let friction_impulse = -tangent_vel.clamp_abs(max_friction_impulse);

                guy.ball.pos += collision.normal * collision.penetration;
                guy.ball.vel += tangent * friction_impulse / guy.mass(&self.config);
                guy.ball.w -= friction_impulse / guy.radius() / guy.mass(&self.config);

                guy.ball.vel -=
                    guy.ball.vel * (delta_time * collision.assets.params.speed_friction).min(1.0);
                guy.ball.w -=
                    guy.ball.w * (delta_time * collision.assets.params.rotation_friction).min(1.0);

                // Stickiness
                guy.stick_force = std::cmp::max_by_key(
                    guy.stick_force,
                    (normal_vel * collision.assets.params.stick_strength)
                        .clamp_abs(collision.assets.params.max_stick_force)
                        * collision.normal,
                    |force| r32(force.len()),
                );

                // Snow layer
                if collision.surface.type_name == "snow" {
                    guy.snow_layer += guy.ball.w.abs() * delta_time * 1e-2;
                }

                {
                    let snow_falloff = ((bounce_impulse.abs()
                        - self.config.snow_falloff_impulse_min)
                        / (self.config.snow_falloff_impulse_max
                            - self.config.snow_falloff_impulse_min))
                        .clamp(0.0, 1.0)
                        * self.config.max_snow_layer
                        * collision.assets.params.snow_falloff;
                    let snow_falloff = snow_falloff.min(guy.snow_layer);
                    guy.snow_layer -= snow_falloff;
                    let fart_type = "normal"; // TODO: not normal?
                    let fart_assets = &assets.farts[fart_type];
                    let farticles = self.farticles.entry(fart_type.to_owned()).or_default();
                    for _ in 0..(100.0 * snow_falloff / self.config.max_snow_layer) as i32 {
                        farticles.push(Farticle {
                            size: 0.6,
                            pos: guy.ball.pos
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
                guy.snow_layer = guy.snow_layer.clamp(0.0, self.config.max_snow_layer);

                if let Some(sound) = &collision.assets.sound {
                    let volume = ((-0.5 + impulse / 2.0) / 2.0).clamp(0.0, 1.0);
                    if volume > 0.0 {
                        let mut effect = sound.effect();
                        effect.set_volume(
                            (self.volume
                                * volume
                                * (1.0
                                    - (guy.ball.pos - self.camera.center).len() / self.camera.fov))
                                .clamp(0.0, 1.0) as f64,
                        );
                        effect.set_speed(sfx_speed);
                        effect.play();
                    }
                }
            } else {
                guy.stick_force = vec2::ZERO;
            }

            // Portals
            for portal in &self.level.portals {
                let is_colliding =
                    |pos: vec2<f32>| -> bool { (pos - portal.pos).len() < self.config.portal.size };
                if !is_colliding(prev_state.pos) && is_colliding(guy.ball.pos) {
                    if let Some(dest) = portal.dest {
                        guy.ball.pos = self.level.portals[dest].pos;
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
                    self.remote_simulation_times
                        .entry(guy.id)
                        .or_insert_with(|| t - 1.0);
                    self.remote_updates
                        .entry(guy.id)
                        .or_default()
                        .push_back((t, guy));
                }
                ServerMessage::Despawn(id) => {
                    self.guys.remove(&id);
                    self.remote_simulation_times.remove(&id);
                    if let Some(updates) = self.remote_updates.get_mut(&id) {
                        updates.clear();
                    }
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
