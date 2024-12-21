use std::{
	fmt,
	str::FromStr,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ByteSize(pub i64);

impl fmt::Display for ByteSize {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let sizes = [("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)];

		let n = self.0.abs();
		for (unit, amount) in sizes {
			if n >= amount {
				let s = format!("{:.2}", self.0 as f64 / amount as f64);
				let s = s.trim_end_matches('0').trim_end_matches('.');
				return write!(f, "{s}{unit}");
			}
		}

		write!(f, "{n}B")
	}
}

impl FromStr for ByteSize {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with('-') {
			return Err("value can't be negative");
		}
		if s.is_empty() {
			return Err("value can't be empty");
		}
		if !s.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
			return Err("value must start with a number");
		}

		if let Ok(n) = s.parse::<u64>() {
			return Ok(Self(i64::try_from(n).unwrap_or(i64::MAX)));
		}

		let Some(i) = s.find(|c: char| c != '.' && !c.is_ascii_digit()) else {
			return match s.parse::<f64>() {
				Ok(f) => Ok(Self(f as i64)),
				Err(_) => Err("value does not contain a number"),
			};
		};

		let n = s[..i]
			.parse::<f64>()
			.map_err(|_| "value does not contain a number")?;
		let amount = match s[i..].trim().to_lowercase().as_str() {
			"t" | "tb" => i64::pow(10, 12),
			"ti" | "tib" => 1 << 40,
			"g" | "gb" => i64::pow(10, 9),
			"gi" | "gib" => 1 << 30,
			"m" | "mb" => i64::pow(10, 6),
			"mi" | "mib" => 1 << 20,
			"k" | "kb" => i64::pow(10, 3),
			"ki" | "kib" => 1 << 10,
			"" | "b" => 1,
			_ => return Err("unrecognized unit"),
		};

		let bytes = n * amount as f64;
		if bytes > i64::MAX as f64 {
			return Err("value is too big");
		}

		Ok(Self(bytes as i64))
	}
}
