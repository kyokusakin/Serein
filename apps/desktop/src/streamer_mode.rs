//! Streamer Mode follows OBS and XSplit while its automatic switch is on, and keeps the window
//! out of screen captures when asked.
use std::{
	sync::mpsc,
	time::{Duration, Instant},
};

/// How often the process list is read while the automatic switch is on.
const INTERVAL: Duration = Duration::from_secs(10);

#[derive(Default)]
pub struct StreamerMode {
	scan: Option<mpsc::Receiver<bool>>,
	next: Option<Instant>,
	broadcasting: bool,
	capture_hidden: bool,
}
impl StreamerMode {
	/// Turns Streamer Mode on when a broadcaster starts and off when the last one exits;
	/// a manual change holds until the next start or exit.
	pub fn poll(
		&mut self,
		streamer: &mut model::streamer_mode::StreamerMode,
		window: &winit::window::Window,
		ctx: &eframe::egui::Context,
	) {
		if streamer.hides_window() != self.capture_hidden {
			self.capture_hidden = streamer.hides_window();
			platform::capture_exclusion::set_hidden(window, self.capture_hidden);
		}
		if !streamer.automatic {
			*self = Self {
				capture_hidden: self.capture_hidden,
				..Self::default()
			};
			return;
		}
		if let Some(scan) = &self.scan {
			match scan.try_recv() {
				Ok(broadcasting) => {
					self.scan = None;
					if broadcasting != self.broadcasting {
						self.broadcasting = broadcasting;
						streamer.enabled = broadcasting;
					}
				}
				Err(mpsc::TryRecvError::Disconnected) => self.scan = None,
				Err(mpsc::TryRecvError::Empty) => {}
			}
		}
		let now = Instant::now();
		if self.scan.is_none() && self.next.is_none_or(|next| now >= next) {
			self.next = Some(now + INTERVAL);
			let (send, receive) = mpsc::channel();
			let wake = ctx.clone();
			std::thread::spawn(move || {
				let broadcasting = platform::processes::running().is_ok_and(|paths| {
					paths
						.iter()
						.any(|path| model::streamer_mode::is_broadcaster(path))
				});
				let _ = send.send(broadcasting);
				wake.request_repaint();
			});
			self.scan = Some(receive);
		}
		if let Some(next) = self.next {
			ctx.request_repaint_after(next.saturating_duration_since(now));
		}
	}
}
