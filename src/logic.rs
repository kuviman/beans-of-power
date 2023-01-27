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
            roll_direction: {
                let mut direction = 0.0;
                if CONTROLS_LEFT
                    .iter()
                    .any(|&key| self.geng.window().is_key_pressed(key))
                {
                    direction += 1.0;
                }
                if CONTROLS_RIGHT
                    .iter()
                    .any(|&key| self.geng.window().is_key_pressed(key))
                {
                    direction -= 1.0;
                }
                direction
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
        for guy in &mut self.guys {
            if (guy.pos - self.level.finish_point).len() < 1.5 {
                guy.finished = true;
            }
            if !guy.touched_a_unicorn {
                for object in &self.level.objects {
                    if (guy.pos - object.pos).len() < 1.5 && object.type_name == "unicorn" {
                        guy.touched_a_unicorn = true;
                        guy.fart_pressure = self.config.max_fart_pressure;
                    }
                }
            }

            if guy.finished {
                guy.fart_pressure = 0.0;
                guy.rot -= delta_time;
                guy.pos = self.level.finish_point
                    + (guy.pos - self.level.finish_point)
                        .normalize_or_zero()
                        .rotate(delta_time)
                        * 1.0;
                continue;
            }

            guy.w += (guy.input.roll_direction.clamp(-1.0, 1.0)
                * self.config.angular_acceleration
                * delta_time)
                .clamp(
                    -(guy.w + self.config.max_angular_speed).max(0.0),
                    (self.config.max_angular_speed - guy.w).max(0.0),
                );
            guy.vel.y -= self.config.gravity * delta_time;

            let mut in_water = false;
            let butt = guy.pos + vec2(0.0, -self.config.guy_radius * 0.9).rotate(guy.rot);
            'tile_loop: for tile in &self.level.tiles {
                for i in 0..3 {
                    let p1 = tile.vertices[i];
                    let p2 = tile.vertices[(i + 1) % 3];
                    if vec2::skew(p2 - p1, guy.pos - p1) < 0.0 {
                        continue 'tile_loop;
                    }
                }
                let relative_vel = guy.vel - tile.flow;
                let flow_direction = tile.flow.normalize_or_zero();
                let relative_vel_along_flow = vec2::dot(flow_direction, relative_vel);
                let params = &self.assets.tiles[&tile.type_name].params;
                let force_along_flow =
                    -flow_direction * relative_vel_along_flow * params.friction_along_flow;
                let friction_force = -relative_vel * params.friction;
                guy.vel +=
                    (force_along_flow + params.additional_force + friction_force) * delta_time;
                guy.w -= guy.w * params.friction * delta_time;
            }
            'tile_loop: for tile in &self.level.tiles {
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

            let could_fart = guy.fart_pressure >= self.config.fart_pressure_released;
            if self.config.fart_continued_force == 0.0 {
                guy.farting = false;
            }
            if guy.input.force_fart {
                if guy.farting {
                    guy.fart_pressure -= delta_time * self.config.fart_continuation_pressure_speed;
                    if guy.fart_pressure < 0.0 {
                        guy.fart_pressure = 0.0;
                        guy.farting = false;
                    }
                } else {
                    guy.fart_pressure += delta_time * self.config.force_fart_pressure_multiplier;
                }
            } else {
                guy.farting = false;
                guy.fart_pressure += delta_time;
            };

            if !guy.farting {
                if let Some(sfx) = self.long_fart_sfx.get_mut(&guy.id) {
                    if sfx.finish_time.is_none() {
                        sfx.finish_time = Some(self.real_time);
                    }
                    let fadeout = (self.real_time - sfx.finish_time.unwrap()) / 0.2;
                    if fadeout >= 1.0 {
                        sfx.sfx.stop();
                        sfx.bubble_sfx.stop();
                        sfx.rainbow_sfx.stop();
                        self.long_fart_sfx.remove(&guy.id);
                    } else {
                        let active_index = if in_water {
                            0
                        } else if guy.touched_a_unicorn {
                            1
                        } else {
                            2
                        };
                        for (index, sfx) in
                            [&mut sfx.bubble_sfx, &mut sfx.rainbow_sfx, &mut sfx.sfx]
                                .into_iter()
                                .enumerate()
                        {
                            if index == active_index {
                                sfx.set_volume(
                                    (self.volume
                                        * (1.0
                                            - (guy.pos - self.camera.center).len()
                                                / self.camera.fov))
                                        .clamp(0.0, 1.0) as f64
                                        * (1.0 - fadeout) as f64,
                                );
                            } else {
                                sfx.set_volume(0.0);
                            }
                        }
                    }
                }
            } else {
                if let Some(sfx) = self.long_fart_sfx.get_mut(&guy.id) {
                    let active_index = if in_water {
                        0
                    } else if guy.touched_a_unicorn {
                        1
                    } else {
                        2
                    };
                    for (index, sfx) in [&mut sfx.bubble_sfx, &mut sfx.rainbow_sfx, &mut sfx.sfx]
                        .into_iter()
                        .enumerate()
                    {
                        if index == active_index {
                            sfx.set_volume(
                                (self.volume
                                    * (1.0
                                        - (guy.pos - self.camera.center).len() / self.camera.fov))
                                    .clamp(0.0, 1.0) as f64,
                            );
                        } else {
                            sfx.set_volume(0.0);
                        }
                    }
                } else {
                    warn!("No sfx for long fart?");
                }
            }

            if guy.farting {
                guy.next_farticle -= delta_time;
                while guy.next_farticle < 0.0 {
                    guy.next_farticle += 1.0 / self.config.long_fart_farticles_per_second;
                    self.farticles.push(Farticle {
                        size: 1.0,
                        pos: butt,
                        vel: guy.vel
                            + vec2(
                                thread_rng().gen_range(0.0..=self.config.farticle_additional_vel),
                                0.0,
                            )
                            .rotate(thread_rng().gen_range(0.0..=2.0 * f32::PI))
                            + vec2(0.0, -self.config.long_fart_farticle_speed).rotate(guy.rot),
                        rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
                        w: thread_rng().gen_range(-self.config.farticle_w..=self.config.farticle_w),
                        color: if in_water {
                            self.config.bubble_fart_color
                        } else if guy.touched_a_unicorn {
                            Hsva::new(thread_rng().gen_range(0.0..1.0), 1.0, 1.0, 0.5).into()
                        } else {
                            self.config.fart_color
                        },
                        t: 1.0,
                    });
                }
                guy.vel += vec2(0.0, self.config.fart_continued_force * delta_time).rotate(guy.rot);
            } else if (guy.fart_pressure >= self.config.fart_pressure_released
                && guy.input.force_fart)
                || guy.fart_pressure >= self.config.max_fart_pressure
            {
                guy.fart_pressure -= self.config.fart_pressure_released;
                guy.farting = true;
                {
                    let mut sfx = self.assets.sfx.long_fart.effect();
                    sfx.set_volume(0.0);
                    sfx.play();
                    let mut bubble_sfx = self.assets.sfx.bubble_long_fart.effect();
                    bubble_sfx.set_volume(0.0);
                    bubble_sfx.play();
                    let mut rainbow_sfx = self.assets.sfx.rainbow_long_fart.effect();
                    rainbow_sfx.set_volume(0.0);
                    rainbow_sfx.play();
                    if let Some(mut sfx) = self.long_fart_sfx.insert(
                        guy.id,
                        LongFartSfx {
                            finish_time: None,
                            sfx,
                            bubble_sfx,
                            rainbow_sfx,
                        },
                    ) {
                        sfx.bubble_sfx.stop();
                        sfx.sfx.stop();
                        sfx.rainbow_sfx.stop();
                    }
                }
                for _ in 0..self.config.farticle_count {
                    self.farticles.push(Farticle {
                        size: 1.0,
                        pos: butt,
                        vel: guy.vel
                            + vec2(
                                thread_rng().gen_range(0.0..=self.config.farticle_additional_vel),
                                0.0,
                            )
                            .rotate(thread_rng().gen_range(0.0..=2.0 * f32::PI)),
                        rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
                        w: thread_rng().gen_range(-self.config.farticle_w..=self.config.farticle_w),
                        color: if in_water {
                            self.config.bubble_fart_color
                        } else if guy.touched_a_unicorn {
                            Hsva::new(thread_rng().gen_range(0.0..1.0), 1.0, 1.0, 0.5).into()
                        } else {
                            self.config.fart_color
                        },
                        t: 1.0,
                    });
                }
                guy.vel += vec2(0.0, self.config.fart_strength).rotate(guy.rot);
                let sounds = if in_water {
                    &self.assets.sfx.bubble_fart
                } else if guy.touched_a_unicorn {
                    &self.assets.sfx.rainbow_fart
                } else {
                    &self.assets.sfx.fart
                };
                let mut effect = sounds.choose(&mut thread_rng()).unwrap().effect();
                effect.set_volume(
                    (self.volume * (1.0 - (guy.pos - self.camera.center).len() / self.camera.fov))
                        .clamp(0.0, 1.0) as f64,
                );
                effect.play();
            } else if !could_fart && guy.fart_pressure >= self.config.fart_pressure_released {
                // Growling stomach recharge
                if Some(guy.id) == self.my_guy {
                    let mut effect = self.assets.sfx.fart_recharge.effect();
                    effect.set_volume(self.volume as f64 * 0.5);
                    effect.play();
                }
                guy.growl_progress = Some(0.0);
            }

            if let Some(growl) = &mut guy.growl_progress {
                *growl += delta_time / self.config.growl_time;
                if *growl >= 1.0 {
                    guy.growl_progress = None;
                }
            }

            guy.pos += guy.vel * delta_time;
            guy.rot += guy.w * delta_time;

            struct Collision<'a> {
                penetration: f32,
                normal: vec2<f32>,
                assets: &'a SurfaceAssets,
            }

            let mut collision_to_resolve = None;
            let mut was_colliding_water = guy.colliding_water;
            guy.colliding_water = false;
            for surface in &self.level.surfaces {
                let v = surface.vector_from(guy.pos);
                let penetration = self.config.guy_radius - v.len();
                if penetration > EPS {
                    let assets = &self.assets.surfaces[&surface.type_name];

                    if surface.type_name == "water" {
                        guy.colliding_water = true;
                        if !was_colliding_water {
                            was_colliding_water = true;
                            if vec2::dot(v, guy.vel).abs() > 0.5 {
                                let mut effect = self.assets.sfx.water_splash.effect();
                                effect.set_volume(
                                    (self.volume
                                        * 0.6
                                        * (1.0
                                            - (guy.pos - self.camera.center).len()
                                                / self.camera.fov))
                                        .clamp(0.0, 1.0) as f64,
                                );
                                effect.play();
                                for _ in 0..30 {
                                    self.farticles.push(Farticle {
                                        size: 0.6,
                                        pos: guy.pos
                                            + v
                                            + vec2(
                                                thread_rng().gen_range(
                                                    -self.config.guy_radius
                                                        ..=self.config.guy_radius,
                                                ),
                                                0.0,
                                            ),
                                        vel: {
                                            let mut v =
                                                vec2(0.0, thread_rng().gen_range(0.0..=1.0))
                                                    .rotate(
                                                        thread_rng().gen_range(
                                                            -f32::PI / 4.0..=f32::PI / 4.0,
                                                        ),
                                                    );
                                            v.y *= 0.3;
                                            v * 2.0
                                        },
                                        rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
                                        w: thread_rng().gen_range(
                                            -self.config.farticle_w..=self.config.farticle_w,
                                        ),
                                        color: self.config.bubble_fart_color,
                                        t: 0.5,
                                    });
                                }
                            }
                        }
                    }

                    if assets.params.non_collidable {
                        continue;
                    }
                    if vec2::dot(v, guy.vel) > EPS {
                        let collision = Collision {
                            penetration,
                            normal: -v.normalize_or_zero(),
                            assets,
                        };
                        collision_to_resolve = std::cmp::max_by_key(
                            collision_to_resolve,
                            Some(collision),
                            |collision| {
                                r32(match collision {
                                    Some(collision) => collision.penetration,
                                    None => -1.0,
                                })
                            },
                        );
                    }
                }
            }
            if let Some(collision) = collision_to_resolve {
                guy.pos += collision.normal * collision.penetration;
                let normal_vel = vec2::dot(guy.vel, collision.normal);
                let tangent = collision.normal.rotate_90();
                let tangent_vel = vec2::dot(guy.vel, tangent) - guy.w * self.config.guy_radius;
                let impulse = (-normal_vel * (1.0 + collision.assets.params.bounciness))
                    .max(-normal_vel + collision.assets.params.min_bounce_vel);
                guy.vel += collision.normal * impulse;
                let max_friction_impulse = normal_vel.abs() * collision.assets.params.friction;
                let friction_impulse = -tangent_vel.clamp_abs(max_friction_impulse);
                guy.vel += tangent * friction_impulse;
                guy.w -= friction_impulse / self.config.guy_radius;
                if let Some(sound) = &collision.assets.sound {
                    let volume = ((-0.5 + impulse / 2.0) / 2.0).clamp(0.0, 1.0);
                    if volume > 0.0 {
                        let mut effect = sound.effect();
                        effect.set_volume(
                            (self.volume
                                * volume
                                * (1.0 - (guy.pos - self.camera.center).len() / self.camera.fov))
                                .clamp(0.0, 1.0) as f64,
                        );
                        effect.play();
                    }
                }
            }
        }
    }

    pub fn handle_connection(&mut self) {
        let messages: Vec<ServerMessage> = match &mut self.connection {
            Some(con) => con.new_messages().collect(),
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
                    if !self.remote_simulation_times.contains_key(&guy.id) {
                        self.remote_simulation_times.insert(guy.id, t - 1.0);
                    }
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
        let new_guy = Guy::new(self.client_id, self.level.spawn_point, true);
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
