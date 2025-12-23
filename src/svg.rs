use glam::Vec2;
use std::fs::write;
use std::path::Path;

fn calculate_pin_positions(pin_count: usize, size: u32, margin: f32) -> Vec<Vec2> {
    let center = Vec2::new(size as f32 / 2.0, size as f32 / 2.0);
    let radius = (size as f32 - 2.0 * margin) / 2.0; // 半径减去留白

    (0..pin_count)
        .map(|i| {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / pin_count as f32;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            Vec2::new(x, y)
        })
        .collect()
}

fn generate_svg(
    line_sequence: &[usize],
    pin_positions: &[Vec2],
    size: u32,
    line_color: &str,
    line_width: f32,
) -> String {
    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <rect width="100%" height="100%" fill="white"/>
"#,
        size, size
    );

    // 绘制线条（连接序列中连续的针）
    for i in 0..line_sequence.len() - 1 {
        let start = line_sequence[i];
        let end = line_sequence[i + 1];
        let p1 = pin_positions[start];
        let p2 = pin_positions[end];

        svg.push_str(&format!(
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
            p1.x, p1.y, p2.x, p2.y, line_color, line_width
        ));
        svg.push('\n');
    }

    svg.push_str("</svg>");
    svg
}

pub fn save_as_svg(
    line_sequence: &[usize],
    pin_count: usize,
    size: u32,
    output_path: &Path,
    line_width: f32,
) -> Result<(), std::io::Error> {
    // 计算针的坐标（留白 10 像素）
    let pin_positions = calculate_pin_positions(pin_count, size, 10.0);
    // 生成SVG内容
    let svg_content = generate_svg(line_sequence, &pin_positions, size, "#000000", line_width);
    // 写入文件
    write(output_path, svg_content)
}
