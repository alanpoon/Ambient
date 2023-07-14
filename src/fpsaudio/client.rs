use ambient_api::components::core::transform::{rotation, translation};
use ambient_api::prelude::*;

#[main]
pub fn main() {
    let firesound = audio::AudioPlayer::new();
    messages::FireSound::subscribe(move |_, msg| {
        let fire_sound_url = asset::url("assets/sound/m4a1.ogg").unwrap();
        let whoshoot = msg.source;
        let listener = player::get_local();
        let pos_shoot = entity::get_component(whoshoot, translation()).unwrap();
        let pos_listen = entity::get_component(listener, translation()).unwrap();
        let rot_listener = entity::get_component(listener, rotation()).unwrap();
        println!("rot_listener: {:?}", rot_listener);
        let distance = (pos_listen - pos_shoot).length();

        let direction = if distance < 0.0001 {
            Vec3::new(0.0, 0.0, 1.0)
        } else {
            (pos_listen - pos_shoot).normalize()
        };
        println!("direction: {:?}", direction);
        let angle = direction.dot(rot_listener * Vec3::new(-1.0, 0.0, 0.0));
        let forward = direction.dot(rot_listener * Vec3::new(0.0, 1.0, 0.0)) > 0.0;
        println!("forward: {}", forward);
        if !forward && distance > 0.0001 {
            firesound.add_one_pole_lpf(3000.);
        } else {
            firesound.add_one_pole_lpf(8000.);
        }

        println!("angle: {}", angle);
        firesound.set_panning(angle);

        firesound.set_amplitude(
            ({
                if distance <= 1.0 {
                    1.0
                } else {
                    1.0 / distance.log2()
                }
            })
            .clamp(0.0, 1.0),
        );
        firesound.play(fire_sound_url);
    });

    let walksound = audio::AudioPlayer::new();

    messages::WalkSound::subscribe(move |_, msg| {
        let fire_sound_url = asset::url("assets/sound/walk.ogg").unwrap();
        let whoshoot = msg.source;
        let listener = player::get_local();
        let pos_shoot = entity::get_component(whoshoot, translation()).unwrap();
        let pos_listen = entity::get_component(listener, translation()).unwrap();
        let rot_listener = entity::get_component(listener, rotation()).unwrap();
        println!("rot_listener: {:?}", rot_listener);
        let distance = (pos_listen - pos_shoot).length();

        let direction = if distance < 0.0001 {
            Vec3::new(0.0, 0.0, 1.0)
        } else {
            (pos_listen - pos_shoot).normalize()
        };
        println!("direction: {:?}", direction);
        let angle = direction.dot(rot_listener * Vec3::new(-1.0, 0.0, 0.0));
        let forward = direction.dot(rot_listener * Vec3::new(0.0, 1.0, 0.0)) > 0.0;
        println!("forward: {}", forward);
        if !forward && distance > 0.0001 {
            walksound.add_one_pole_lpf(3000.);
        } else {
            walksound.add_one_pole_lpf(8000.);
        }

        println!("angle: {}", angle);
        walksound.set_panning(angle);

        walksound.set_amplitude(
            ({
                if distance <= 1.0 {
                    1.0
                } else {
                    1.0 / distance.log2()
                }
            })
            .clamp(0.0, 1.0),
        );
        walksound.play(fire_sound_url);
    });
}
