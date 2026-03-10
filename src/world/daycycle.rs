use bevy::prelude::*;
use bevy_firefly::prelude::*;

const MIDNIGHT: f64 = 86400.;
const SUNRISE: f64 = 25200.;
const SUNSET: f64 = 68400.;

#[derive(Resource)]
pub struct GameTime {
    time: f64, // measured in seconds
    speedup: f64,
    start_time: f64,
}

impl GameTime {
    pub fn default() -> Self {
        Self { time: 0., speedup: 2000., start_time: SUNRISE - 1000.} // minute long days
    }
    pub fn is_day(&self) -> bool {
        self.time > SUNRISE && self.time < SUNSET
    }
    pub fn is_night(&self) -> bool {
        !self.is_day()
    }
}

pub fn advance_game_time(
    mut game_time: ResMut<GameTime>,
    engine_time: Res<Time>
) {
    game_time.time = (engine_time.elapsed_secs_f64() * game_time.speedup + game_time.start_time) % 86400.;
    info!(game_time.time)
}

pub fn update_ambient_light(
    game_time: Res<GameTime>,
    player_query: Query<&mut FireflyConfig, With<crate::player::player_camera::PlayerCamera>>
) {
    for mut light_config in player_query {
        if game_time.is_night() {
            light_config.ambient_color = Color::hsl(199., 0.437, 0.23)
        }
        else {
            light_config.ambient_color = Color::hsl(52., 1.0, 0.961)
        }
    }
}