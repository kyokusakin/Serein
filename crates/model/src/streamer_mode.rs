//! Discord's Streamer Mode: device-local switches that hide personal details while broadcasting.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct StreamerMode {
	pub enabled: bool,
	/// Turn on while OBS or XSplit runs and off again when it closes.
	pub automatic: bool,
	/// Email, connected accounts and notes.
	pub hide_personal: bool,
	pub hide_invites: bool,
	pub disable_sounds: bool,
	pub disable_notifications: bool,
	/// Exclude this app's windows from screen capture where the platform supports it.
	pub hide_window: bool,
}
impl Default for StreamerMode {
	fn default() -> Self {
		Self {
			enabled: false,
			automatic: true,
			hide_personal: true,
			hide_invites: true,
			disable_sounds: true,
			disable_notifications: true,
			hide_window: false,
		}
	}
}
impl StreamerMode {
	pub fn hides_personal(self) -> bool {
		self.enabled && self.hide_personal
	}
	pub fn hides_invites(self) -> bool {
		self.enabled && self.hide_invites
	}
	pub fn mutes_sounds(self) -> bool {
		self.enabled && self.disable_sounds
	}
	pub fn mutes_notifications(self) -> bool {
		self.enabled && self.disable_notifications
	}
	pub fn hides_window(self) -> bool {
		self.enabled && self.hide_window
	}
}

/// Whether an executable path belongs to OBS Studio or XSplit Broadcaster.
pub fn is_broadcaster(path: &str) -> bool {
	let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
	let name = name.to_ascii_lowercase();
	let stem = name.strip_suffix(".exe").unwrap_or(&name);
	matches!(stem, "obs" | "obs64" | "obs32" | "xsplit.core")
}

#[cfg(test)]
mod tests {
	#[test]
	fn broadcasters_match_by_executable_name_only() {
		assert!(super::is_broadcaster(
			r"C:\Program Files\obs-studio\bin\64bit\obs64.exe"
		));
		assert!(super::is_broadcaster(
			"/Applications/OBS.app/Contents/MacOS/OBS"
		));
		assert!(super::is_broadcaster(
			r"C:\Program Files\XSplit\Broadcaster\XSplit.Core.exe"
		));
		assert!(!super::is_broadcaster("/usr/bin/jobs"));
		assert!(!super::is_broadcaster(r"C:\obs\game.exe"));
	}
}
