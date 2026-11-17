//! Persistent verification codes: one voice encryption identity per account, kept in the OS
//! credential store so friends verify this device once instead of every call.
use discord_voice::Identity;
use model::Id;
use std::sync::{Arc, mpsc};

type Loaded = Result<Arc<Identity>, platform::CredentialError>;

#[derive(Default)]
pub struct Kept {
	/// Account and setting the current identity, job and status answer.
	scope: Option<(Id, bool)>,
	identity: Option<Arc<Identity>>,
	job: Option<mpsc::Receiver<Loaded>>,
	pub status: &'static str,
}
impl Kept {
	/// Loads or creates the account's identity while the setting is on, and deletes it when
	/// the setting is turned off.
	pub fn poll(&mut self, account: Option<Id>, enabled: bool, ctx: &eframe::egui::Context) {
		if let Some(job) = &self.job {
			match job.try_recv() {
				Ok(Ok(identity)) => {
					self.identity = Some(identity);
					self.job = None;
				}
				Ok(Err(_)) | Err(mpsc::TryRecvError::Disconnected) => {
					self.job = None;
					self.status = "settings-persistent-verification-codes-unavailable";
				}
				Err(mpsc::TryRecvError::Empty) => {}
			}
		}
		let scope = account.map(|account| (account, enabled));
		if scope == self.scope {
			return;
		}
		let previous = std::mem::replace(&mut self.scope, scope);
		self.identity = None;
		self.job = None;
		self.status = "";
		let Some((account, enabled)) = scope else {
			return;
		};
		if enabled {
			let (send, receive) = mpsc::channel();
			let wake = ctx.clone();
			std::thread::spawn(move || {
				let _ = send.send(load_or_create(account));
				wake.request_repaint();
			});
			self.job = Some(receive);
		} else if previous == Some((account, true)) {
			std::thread::spawn(move || {
				let _ = platform::forget_voice_identity(account);
			});
		}
	}
	pub fn identity(&self) -> Option<Arc<Identity>> {
		self.identity.clone()
	}
}

fn load_or_create(account: Id) -> Loaded {
	if let Some(key) = platform::load_voice_identity(account)?
		&& let Some(identity) = Identity::from_private(&key)
	{
		return Ok(identity);
	}
	let identity = Identity::generate();
	platform::save_voice_identity(account, &identity.private())?;
	Ok(identity)
}
