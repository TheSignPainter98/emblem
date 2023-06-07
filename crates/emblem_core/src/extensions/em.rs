use derive_new::new;
use mlua::{MetaMethod, UserData};

#[derive(new)]
pub(crate) struct Em {}

impl UserData for Em {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("version", |lua, _| lua.create_userdata(Version::new()));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    fn new() -> Self {
        let mut parts = Self::raw().split('.');
        Self {
            major: parts.next().unwrap().parse().unwrap(),
            minor: parts.next().unwrap().parse().unwrap(),
            patch: parts.next().unwrap().parse().unwrap(),
        }
    }

    fn raw() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

impl UserData for Version {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("major", |_, this| Ok(this.major));
        fields.add_field_method_get("minor", |_, this| Ok(this.minor));
        fields.add_field_method_get("patch", |_, this| Ok(this.patch));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::ToString, |_, _, ()| {
            Ok(format!("<version {}>", Version::raw()))
        });
    }
}
