use derive_new::new;
use mlua::UserData;

#[derive(new)]
pub(crate) struct Em {}

impl UserData for Em {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("version", |_, _| Ok(env!("CARGO_PKG_VERSION")));
    }
}
