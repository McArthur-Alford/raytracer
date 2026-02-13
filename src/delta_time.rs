use std::time::Instant;

use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct DeltaTime(pub f64);
