use serde_json::Value;

#[test]
fn settings_use_required_group_root_and_mask_the_api_key() {
    let groups: Value = serde_json::from_str(include_str!("../res/settings.json")).unwrap();
    let mut keys = Vec::new();
    for group in groups.as_array().unwrap() {
        assert_eq!(group["type"], "group");
        for setting in group["items"].as_array().unwrap() {
            keys.push(setting["key"].as_str().unwrap());
            assert_eq!(setting["autocapitalizationType"], 0);
            if setting["key"] == "api_key" {
                assert_eq!(setting["secure"], true);
                assert!(setting.get("default").is_none());
            }
        }
    }
    assert_eq!(keys, ["server_url", "api_key"]);
}

#[test]
fn icon_has_required_dimensions_and_opaque_rgb_encoding() {
    let png = include_bytes!("../res/icon.png");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&png[12..16], b"IHDR");
    assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 128);
    assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 128);
    assert_eq!(png[25], 2); // 알파 채널이 없는 RGB PNG.
    let mut offset = 8;
    while offset + 12 <= png.len() {
        let length = u32::from_be_bytes(png[offset..offset + 4].try_into().unwrap()) as usize;
        assert_ne!(&png[offset + 4..offset + 8], b"tRNS");
        offset += length + 12;
    }
}
