pub(crate) fn ios_video_player_allows_arbitrary_loads(manifest: &serde_json::Value) -> bool {
    manifest
        .get("app-plus")
        .and_then(|value| value.get("distribute"))
        .and_then(|value| value.get("ios"))
        .and_then(|value| value.get("NSAppTransportSecurity"))
        .and_then(|value| {
            value
                .as_object()
                .and_then(|map| map.get("NSAllowsArbitraryLoads"))
                .or(Some(value))
        })
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}
