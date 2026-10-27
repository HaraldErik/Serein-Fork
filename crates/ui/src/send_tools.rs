//! Device-local rewrites applied to a draft when it is sent (Equicord ClearURLs and
//! SendTimestamps equivalents). Code spans stay untouched, except that a code span holding only
//! a clock time such as `` `18:30` `` becomes a Discord timestamp.

/// Query parameters that only track the click, removed from every host.
const TRACKING: &[&str] = &[
	"fbclid",
	"gclid",
	"gclsrc",
	"dclid",
	"gbraid",
	"wbraid",
	"msclkid",
	"mc_cid",
	"mc_eid",
	"igshid",
	"igsh",
	"yclid",
	"ttclid",
	"twclid",
	"li_fat_id",
	"srsltid",
	"_hsenc",
	"_hsmi",
	"mkt_tok",
	"_ga",
	"_gl",
	"spm",
	"ref_src",
	"ref_url",
	"oly_anon_id",
	"oly_enc_id",
	"vero_id",
	"rb_clickid",
	"s_cid",
	"wickedid",
];

/// Parameters that only track on specific hosts (elsewhere they can carry meaning).
fn host_tracking(host: &str, key: &str, value: &str) -> bool {
	let host = host.strip_prefix("www.").unwrap_or(host);
	let host = host.strip_prefix("m.").unwrap_or(host);
	match host {
		"youtube.com" | "music.youtube.com" | "youtu.be" => {
			key == "si" || key == "pp" || (key == "feature" && value == "share")
		}
		"open.spotify.com" => key == "si" || key == "context",
		"x.com" | "twitter.com" | "fxtwitter.com" | "vxtwitter.com" => key == "s" || key == "t",
		"tiktok.com" | "vm.tiktok.com" => {
			matches!(
				key,
				"_r" | "_t" | "is_from_webapp" | "sender_device" | "sender_web_id"
			)
		}
		"reddit.com" | "old.reddit.com" => key == "share_id",
		"instagram.com" => key == "igsh" || key == "img_index",
		_ => false,
	}
}

/// Byte ranges of inline/fenced code, matched like Discord: a run of N backticks closes at
/// the next run of exactly N; an unmatched run is literal text.
fn code_spans(text: &str) -> Vec<(usize, usize, usize)> {
	let bytes = text.as_bytes();
	let mut runs = Vec::new();
	let mut index = 0;
	while index < bytes.len() {
		if bytes[index] == b'`' {
			let start = index;
			while index < bytes.len() && bytes[index] == b'`' {
				index += 1;
			}
			runs.push((start, index));
		} else {
			index += 1;
		}
	}
	let mut spans = Vec::new();
	let mut open = 0;
	while open < runs.len() {
		let (start, end) = runs[open];
		let width = end - start;
		if let Some(close) = (open + 1..runs.len()).find(|&i| runs[i].1 - runs[i].0 == width) {
			spans.push((start, runs[close].1, width));
			open = close + 1;
		} else {
			open += 1;
		}
	}
	spans
}

/// Removes tracking parameters from every `http(s)` link outside code.
pub(crate) fn clear_urls(text: &str) -> String {
	rewrite_outside_code(text, |segment, out| {
		let mut rest = segment;
		while let Some(at) = find_url_start(rest) {
			out.push_str(&rest[..at]);
			let tail = &rest[at..];
			let length = tail
				.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '|'))
				.unwrap_or(tail.len());
			let (url, trailing) = split_trailing_punctuation(&tail[..length]);
			out.push_str(&clean_url(url));
			out.push_str(trailing);
			rest = &tail[length..];
		}
		out.push_str(rest);
	})
}

fn find_url_start(text: &str) -> Option<usize> {
	let http = text.find("http://");
	let https = text.find("https://");
	match (http, https) {
		(Some(a), Some(b)) => Some(a.min(b)),
		(a, b) => a.or(b),
	}
}

/// Sentence punctuation after a link is not part of it; a `)` is kept only when balanced.
fn split_trailing_punctuation(url: &str) -> (&str, &str) {
	let mut end = url.len();
	while let Some(last) = url[..end].chars().next_back() {
		let strip = match last {
			'.' | ',' | '!' | '?' | ':' | ';' | '\'' | '*' | '_' | '~' => true,
			')' => url[..end].matches('(').count() < url[..end].matches(')').count(),
			_ => false,
		};
		if !strip {
			break;
		}
		end -= last.len_utf8();
	}
	(&url[..end], &url[end..])
}

fn clean_url(url: &str) -> String {
	let Some(query_at) = url.find('?') else {
		return url.to_owned();
	};
	let (base, rest) = url.split_at(query_at);
	let (query, fragment) = match rest.find('#') {
		Some(hash) => (&rest[1..hash], &rest[hash..]),
		None => (&rest[1..], ""),
	};
	let host = base
		.split_once("://")
		.map_or("", |(_, after)| after)
		.split(['/', ':'])
		.next()
		.unwrap_or("")
		.to_ascii_lowercase();
	let kept: Vec<&str> = query
		.split('&')
		.filter(|pair| !pair.is_empty())
		.filter(|pair| {
			let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
			let key = key.to_ascii_lowercase();
			!(key.starts_with("utm_")
				|| TRACKING.contains(&key.as_str())
				|| host_tracking(&host, &key, value))
		})
		.collect();
	if kept.len() == query.split('&').filter(|pair| !pair.is_empty()).count() {
		return url.to_owned();
	}
	let mut cleaned = base.to_owned();
	if !kept.is_empty() {
		cleaned.push('?');
		cleaned.push_str(&kept.join("&"));
	}
	cleaned.push_str(fragment);
	cleaned
}

/// Calls `edit` on every stretch outside code; code spans are copied unchanged.
fn rewrite_outside_code(text: &str, mut edit: impl FnMut(&str, &mut String)) -> String {
	let mut out = String::with_capacity(text.len());
	let mut offset = 0;
	for (start, end, _) in code_spans(text) {
		edit(&text[offset..start], &mut out);
		out.push_str(&text[start..end]);
		offset = end;
	}
	edit(&text[offset..], &mut out);
	out
}

/// Turns single-backtick code spans that hold only a clock time (`` `18:30` ``, `` `6:30 pm` ``)
/// into `<t:…:t>` for today in the sender's zone, so every reader sees their own local time.
pub(crate) fn send_timestamps(text: &str, today: time::OffsetDateTime) -> String {
	let mut out = String::with_capacity(text.len());
	let mut offset = 0;
	for (start, end, width) in code_spans(text) {
		if width != 1 {
			continue;
		}
		let Some(clock) = parse_clock(&text[start + 1..end - 1]) else {
			continue;
		};
		let at = today.replace_time(clock).unix_timestamp();
		out.push_str(&text[offset..start]);
		out.push_str(&format!("<t:{at}:t>"));
		offset = end;
	}
	out.push_str(&text[offset..]);
	out
}

fn parse_clock(value: &str) -> Option<time::Time> {
	let value = value.trim();
	let lower = value.to_ascii_lowercase();
	let (clock, meridiem) = if let Some(clock) = lower.strip_suffix("am") {
		(clock.trim_end(), Some(false))
	} else if let Some(clock) = lower.strip_suffix("pm") {
		(clock.trim_end(), Some(true))
	} else {
		(lower.as_str(), None)
	};
	let (hour, minute) = clock.split_once(':')?;
	if !(1..=2).contains(&hour.len())
		|| minute.len() != 2
		|| !hour
			.bytes()
			.chain(minute.bytes())
			.all(|b| b.is_ascii_digit())
	{
		return None;
	}
	let (mut hour, minute): (u8, u8) = (hour.parse().ok()?, minute.parse().ok()?);
	if let Some(pm) = meridiem {
		if !(1..=12).contains(&hour) {
			return None;
		}
		hour = match (hour, pm) {
			(12, false) => 0,
			(12, true) => 12,
			(hour, true) => hour + 12,
			(hour, false) => hour,
		};
	}
	time::Time::from_hms(hour, minute, 0).ok()
}

/// Marks a new message silent (no push notification) unless it already is.
pub(crate) fn make_silent(text: &str) -> String {
	if model::message_options::content(text).1 {
		text.to_owned()
	} else {
		format!("@silent {text}")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn tracking_parameters_are_removed_but_meaningful_ones_and_code_stay() {
		for (input, expected) in [
			(
				"look https://example.com/a?utm_source=x&id=5&fbclid=abc#top.",
				"look https://example.com/a?id=5#top.",
			),
			(
				"https://youtu.be/dQw4w9WgXcQ?si=TRACK&t=42",
				"https://youtu.be/dQw4w9WgXcQ?t=42",
			),
			(
				"https://www.youtube.com/watch?v=abc&pp=xyz&feature=share",
				"https://www.youtube.com/watch?v=abc",
			),
			(
				"(https://x.com/u/status/1?s=20&t=abc)",
				"(https://x.com/u/status/1)",
			),
			(
				"https://open.spotify.com/track/1?si=abc",
				"https://open.spotify.com/track/1",
			),
			// `t` and `s` only track on X/Twitter; elsewhere they may be real parameters.
			(
				"https://example.com/search?s=rust&t=1",
				"https://example.com/search?s=rust&t=1",
			),
			(
				"https://en.wikipedia.org/wiki/Rust_(language)?utm_medium=x",
				"https://en.wikipedia.org/wiki/Rust_(language)",
			),
			(
				"`https://a.io/?utm_source=x` and ```\nhttps://a.io/?utm_source=x\n```",
				"`https://a.io/?utm_source=x` and ```\nhttps://a.io/?utm_source=x\n```",
			),
			("no links here ?utm_source=x", "no links here ?utm_source=x"),
		] {
			assert_eq!(clear_urls(input), expected, "{input}");
		}
	}

	#[test]
	fn clock_code_spans_become_local_timestamps() {
		let offset = time::UtcOffset::from_hms(2, 0, 0).unwrap();
		let today = time::OffsetDateTime::from_unix_timestamp(1_790_000_000)
			.unwrap()
			.to_offset(offset);
		let at = |hour, minute| {
			today
				.replace_time(time::Time::from_hms(hour, minute, 0).unwrap())
				.unix_timestamp()
		};
		assert_eq!(
			send_timestamps("meet `18:30` or `6:30 pm`, not `25:00`/`9:5`", today),
			format!(
				"meet <t:{}:t> or <t:{}:t>, not `25:00`/`9:5`",
				at(18, 30),
				at(18, 30)
			)
		);
		assert_eq!(
			send_timestamps("`12:00am` `12:15PM` ``18:30``", today),
			format!("<t:{}:t> <t:{}:t> ``18:30``", at(0, 0), at(12, 15))
		);
		// A unix timestamp is independent of zone: 18:30 at +02:00 is 16:30 UTC.
		assert_eq!(at(18, 30) % 86_400, (16 * 3600 + 30 * 60) as i64);
	}

	#[test]
	fn silent_prefix_is_added_once() {
		assert_eq!(make_silent("hi"), "@silent hi");
		assert_eq!(make_silent("@silent hi"), "@silent hi");
	}
}
