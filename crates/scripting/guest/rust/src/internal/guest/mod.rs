use once_cell::sync::Lazy;

use super::executor::{FrameState, EXECUTOR};
use crate::Components;

wit_bindgen_guest_rust::export!("src/internal/guest.wit");

mod conversion;

struct Guest;
impl guest::Guest for Guest {
    fn init() {
        Lazy::force(&EXECUTOR);
    }

    fn exec(
        ctx: guest::RunContext,
        event_name: String,
        components: Vec<(u32, guest::ComponentType)>,
    ) {
        use conversion::GuestConvert;

        let components = Components(
            components
                .into_iter()
                .map(|(id, ct)| (id, ct.guest_convert()))
                .collect(),
        );

        EXECUTOR.execute(
            FrameState::new(ctx.time, ctx.frametime),
            event_name.as_str(),
            &components,
        );
    }
}
