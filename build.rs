// Emits the app icon (RT_ICON + RT_GROUP_ICON) into the executable.
// The icon group keeps the stable ordinal 156 shared with the tray icon
// (tray_icon::Icon::from_resource(156, None)).
fn main() {
    embed_resource::compile("resources/icon.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}