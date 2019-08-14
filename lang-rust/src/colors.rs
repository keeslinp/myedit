use types::Color;
pub fn get_color_from_tag(tag: &str) -> Option<Color> {
    match tag {
        "function" => Some(Color {
            r: 128,
            g: 160,
            b: 255,
        }),
        "variable" => Some(Color {
            r: 128,
            g: 160,
            b: 255,
        }),
        "string" => Some(Color {
            r: 207,
            g: 207,
            b: 176,
        }),
        "keyword" | "keyword.control" => Some(Color {
            r: 133,
            g: 220,
            b: 133,
        }),
        "comment" => Some(Color {
            r: 142,
            g: 144,
            b: 140,
        }),
        "attribute" => Some(Color {
            r: 200,
            g: 40,
            b: 41,
        }),
        "type" => Some(Color {
            r: 66,
            g: 113,
            b: 174,
        }),
        _ => None,
    }
}
