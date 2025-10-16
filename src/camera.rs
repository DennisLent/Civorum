use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::prelude::EulerRot;
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

pub fn transform_from(camera: &OrbitCamera) -> Transform {
    let rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
    let offset = rotation * Vec3::Z * camera.distance;
    let position = camera.target + offset;
    Transform::from_translation(position).looking_at(camera.target, Vec3::Y)
}

pub fn orbit_camera_controls(
    time: Res<Time>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        mouse_delta += event.delta;
    }

    let mut scroll_delta = 0.0;
    for event in mouse_wheel_events.read() {
        scroll_delta += event.y;
    }

    for (mut camera, mut transform) in &mut query {
        // Rotate with right mouse
        if buttons.pressed(MouseButton::Right) {
            camera.yaw -= mouse_delta.x * 0.005;
            camera.pitch += mouse_delta.y * 0.005;
            camera.pitch = camera
                .pitch
                .clamp(-std::f32::consts::FRAC_PI_2 * 0.99, std::f32::consts::FRAC_PI_2 * 0.99);
        }

        // Zoom with wheel
        if scroll_delta.abs() > f32::EPSILON {
            let s = 1.0 - scroll_delta * 0.1;
            camera.distance = (camera.distance * s).clamp(camera.min_distance, camera.max_distance);
            println!("{}", camera.distance);
        }

        // Pan with WASD/Arrows
        let mut pan_input = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            pan_input.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            pan_input.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            pan_input.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            pan_input.x += 1.0;
        }

        // Pan with middle mouse drag
        if buttons.pressed(MouseButton::Middle) {
            // Convert screen delta to world movement at current yaw
            let rotation = Quat::from_rotation_y(camera.yaw);
            // Scale movement by distance for consistent feel
            let movement =
                rotation * Vec3::new(-mouse_delta.x, 0.0, mouse_delta.y) * 0.005 * camera.distance;
            camera.target += Vec3::new(movement.x, 0.0, movement.z);
        }

        if pan_input.length_squared() > 0.0 {
            let rotation = Quat::from_rotation_y(camera.yaw);
            let movement = rotation * pan_input.normalize() * camera.distance * 0.5 * time.delta_secs();
            camera.target += Vec3::new(movement.x, 0.0, movement.z);
        }

        *transform = transform_from(&camera);
    }
}

