//! Plays soundboard sounds others play in this device's call, downloaded from Discord's
//! documented soundboard CDN path, and sends the entrance sound after joining.
use crate::notification_sounds::{Clip, Sounds};
use model::{Id, soundboard::MAX_FILE_BYTES};
use std::{collections::VecDeque, sync::Arc, time::Duration};

/// Downloaded sounds kept for replays.
const CACHED: usize = 16;

/// Sound being fetched, the volume to play it at, and the fetch.
type Download = (Id, u8, tokio::task::JoinHandle<Option<Vec<u8>>>);

#[derive(Default)]
pub struct Soundboard {
	player: Sounds,
	cache: VecDeque<(Id, Arc<[u8]>)>,
	download: Option<Download>,
}
impl Soundboard {
	/// Plays queued sounds at the effect volume scaled by the account's soundboard volume,
	/// except sounds from participants muted for soundboard on this device.
	pub fn poll(
		&mut self,
		state: &mut client_core::State,
		deafened: bool,
		muted: &[u64],
		runtime: &tokio::runtime::Handle,
		ctx: &eframe::egui::Context,
	) {
		if self
			.download
			.as_ref()
			.is_some_and(|(_, _, task)| task.is_finished())
			&& let Some((sound, volume, task)) = self.download.take()
			&& let Some(Ok(Some(bytes))) = futures_util::FutureExt::now_or_never(task)
		{
			let bytes: Arc<[u8]> = bytes.into();
			if self.cache.len() == CACHED {
				self.cache.pop_front();
			}
			self.cache.push_back((sound, bytes.clone()));
			if !deafened {
				self.player.play_clip(Clip::File(bytes), volume, ctx);
			}
		}
		if self.download.is_some() {
			ctx.request_repaint_after(Duration::from_millis(100));
		}
		let account = state
			.soundboard
			.settings()
			.map_or(100.0, |settings| settings.volume);
		while let Some(effect) = state.take_soundboard_effect() {
			let volume = (effect.volume * account).round().clamp(0.0, 100.0) as u8;
			if deafened || volume == 0 || muted.contains(&effect.user.0) {
				continue;
			}
			if let Some((_, bytes)) = self.cache.iter().find(|(id, _)| *id == effect.sound) {
				self.player
					.play_clip(Clip::File(bytes.clone()), volume, ctx);
				continue;
			}
			// A newer sound replaces a download still in flight.
			if let Some((_, _, task)) = self.download.take() {
				task.abort();
			}
			self.download = Some((effect.sound, volume, runtime.spawn(download(effect.sound))));
			ctx.request_repaint_after(Duration::from_millis(100));
		}
	}
	pub fn stop(&mut self) {
		self.player.stop();
		if let Some((_, _, task)) = self.download.take() {
			task.abort();
		}
	}
}

async fn download(sound: Id) -> Option<Vec<u8>> {
	let url = format!("https://cdn.discordapp.com/soundboard-sounds/{sound}");
	let client = wreq::Client::builder()
		.https_only(true)
		.no_proxy()
		.redirect(wreq::redirect::Policy::none())
		.timeout(Duration::from_secs(15))
		.connect_timeout(Duration::from_secs(5))
		.build()
		.ok()?;
	let response = client_core::fingerprint::discord_request(client.get(&url), &url)
		.header(wreq::header::ACCEPT_ENCODING, "identity")
		.send()
		.await
		.ok()?;
	if response.status() != wreq::StatusCode::OK
		|| response
			.content_length()
			.is_some_and(|size| size > MAX_FILE_BYTES as u64)
	{
		return None;
	}
	let mut bytes = Vec::new();
	let mut body = std::pin::pin!(response.bytes_stream());
	while let Some(chunk) = futures_util::StreamExt::next(&mut body).await {
		let chunk = chunk.ok()?;
		if bytes.len() + chunk.len() > MAX_FILE_BYTES {
			return None;
		}
		bytes.extend_from_slice(&chunk);
	}
	(!bytes.is_empty()).then_some(bytes)
}
