// 保留两位小数
pub fn f64_keep_two(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// 添加百分比号
pub fn f64_to_percentage(value: f64) -> String {
    format!("{:.2}%", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_to_str() {
        assert_eq!(f64_keep_two(123.4567), 123.46);
        assert_eq!(f64_keep_two(0.1234), 0.12);
        assert_eq!(f64_keep_two(0.0), 0.00);
    }

    #[test]
    fn test_f64_to_percentage() {
        assert_eq!(f64_to_percentage(12.3456), "12.35%");
        assert_eq!(f64_to_percentage(0.1234), "0.12%");
        assert_eq!(f64_to_percentage(0.0), "0.00%");
    }
}