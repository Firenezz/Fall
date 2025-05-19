use bevy::{input::{mouse::MouseWheel, ButtonInput}, math::Vec3, prelude::*, render::camera::Camera};

// A simple camera system for moving and zooming the camera.
#[allow(dead_code)]
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Projection), With<Camera>>,
) {
    for (mut transform, mut projection) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyZ) {
            match projection.as_mut() {
                Projection::Orthographic(ortho) => {
                    ortho.scale += 0.1;
                }
                _ => {}
            }
        }

        if keyboard_input.pressed(KeyCode::KeyX) {
            match projection.as_mut() {
                Projection::Orthographic(ortho) => {
                    ortho.scale -= 0.1;
                }
                _ => {}
            }
        }

        match projection.as_mut() {
            Projection::Orthographic(ortho) if ortho.scale < 0.5 => {
                ortho.scale = 0.5;
            }
            _ => {}
        }

        

        let z = transform.translation.z;
        transform.translation += time.delta_secs() * direction * 500.;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}

pub fn zoom_scroll(
    mut evr_scroll: EventReader<MouseWheel>,
    mut query: Query<&mut Projection , With<Camera>>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                for mut ortho in query.iter_mut() {
                    match ortho.as_mut() {
                        Projection::Orthographic(ortho) => {
                            ortho.scale += ev.y * 0.1;
                        }
                        _ => {}
                    }
                }
            }
            MouseScrollUnit::Pixel => {
                for mut ortho in query.iter_mut() {
                    match ortho.as_mut() {
                        Projection::Orthographic(ortho) => {
                            ortho.scale += ev.y * 0.01;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}