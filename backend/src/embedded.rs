use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$FRONTEND_ASSETS"]
pub struct Assets;
