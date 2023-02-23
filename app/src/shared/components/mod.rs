use crate::shared;

pub(crate) fn init() -> anyhow::Result<()> {
    ambient_app::init_all_components();
    ambient_network::init_all_components();
    ambient_physics::init_all_components();
    ambient_wasm::shared::init_components();
    ambient_decals::init_components();
    ambient_world_audio::init_components();
    ambient_primitives::init_components();
    ambient_project::init_components();
    ambient_prefab::init_components();
    ambient_sky::init_components();
    ambient_water::init_components();

    shared::player::init_all_components();

    Ok(())
}

#[cfg(not(feature = "production"))]
pub(crate) mod dev;