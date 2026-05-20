pub fn input_class() -> &'static str {
    "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm \
     placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 \
     sm:text-sm transition duration-150"
}

pub fn get_token() -> Option<String> {
    leptos::window()
        .local_storage()
        .ok()?
        .and_then(|s| s.get_item("jwt_token").ok()?)
}

pub fn format_km(km: i32) -> String {
    let s = km.to_string();
    let chars: Vec<char> = s.chars().collect();
    let formatted = chars
        .rchunks(3)
        .rev()
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\u{202F}");
    format!("{} km", formatted)
}
