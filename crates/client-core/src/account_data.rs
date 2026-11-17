//! The account's subscriptions, boosts, gifts, billing and connections, read each time their
//! settings page opens. Only the latest page's listing is kept.
use crate::{Command, State, auth::Failure};
use model::account_data::{Data, Section};

#[derive(Default)]
pub struct AccountData {
	data: Option<Data>,
	error: Option<(Section, Failure)>,
	pending: Option<(Section, u64)>,
	sequence: u64,
}
impl AccountData {
	pub fn get(&self, section: Section) -> Option<&Data> {
		self.data.as_ref().filter(|data| data.section() == section)
	}
	pub fn error(&self, section: Section) -> Option<Failure> {
		self.error
			.filter(|(failed, _)| *failed == section)
			.map(|(_, failure)| failure)
	}
	pub fn loading(&self, section: Section) -> bool {
		self.pending.is_some_and(|(pending, _)| pending == section)
	}
}

impl State {
	/// Reads the listing behind a settings page, replacing any earlier read still in flight.
	pub fn read_account_data(&mut self, section: Section) -> Option<Command> {
		let account = &mut self.account_data;
		if account.loading(section) {
			return None;
		}
		account.error = None;
		account.sequence = account.sequence.wrapping_add(1);
		account.pending = Some((section, account.sequence));
		Some(Command::AccountData {
			section,
			request: account.sequence,
		})
	}
	pub(crate) fn apply_account_data(&mut self, request: u64, result: Result<Data, Failure>) {
		let account = &mut self.account_data;
		let Some((section, _)) = account.pending.take_if(|(_, pending)| *pending == request) else {
			return;
		};
		match result {
			Ok(data) if data.section() == section && data.valid() => {
				account.data = Some(data);
				account.error = None;
			}
			Ok(_) => account.error = Some((section, Failure::Protocol)),
			Err(failure) => account.error = Some((section, failure)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn only_the_latest_page_read_applies() {
		let mut state = State::default();
		let Some(Command::AccountData { request: gifts, .. }) =
			state.read_account_data(Section::Gifts)
		else {
			panic!("opening a page reads its listing");
		};
		assert!(state.read_account_data(Section::Gifts).is_none());
		let Some(Command::AccountData {
			request: billing, ..
		}) = state.read_account_data(Section::Billing)
		else {
			panic!("another page replaces the read in flight");
		};
		state.apply_account_data(gifts, Ok(Data::Gifts(Vec::new())));
		assert!(state.account_data.get(Section::Gifts).is_none());
		state.apply_account_data(billing, Ok(Data::Boosts(Vec::new())));
		assert_eq!(
			state.account_data.error(Section::Billing),
			Some(Failure::Protocol)
		);
		let Some(Command::AccountData { request, section }) =
			state.read_account_data(Section::Billing)
		else {
			panic!("reading again after a failure");
		};
		assert_eq!(section, Section::Billing);
		state.apply_account_data(
			request,
			Ok(Data::Billing {
				sources: Vec::new(),
				payments: Vec::new(),
			}),
		);
		assert!(state.account_data.get(Section::Billing).is_some());
		assert!(state.account_data.error(Section::Billing).is_none());
	}
}
