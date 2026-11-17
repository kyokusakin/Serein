//! Discovery: the featured public servers, or a search, read each time it is asked for.
use crate::{Command, State, auth::Failure};
use model::discovery::{Page, valid_query};

#[derive(Default)]
pub struct Discovery {
	pub open: bool,
	pub page: Option<Page>,
	pub error: Option<Failure>,
	pending: Option<(String, u64)>,
	sequence: u64,
}
impl Discovery {
	pub fn loading(&self) -> bool {
		self.pending.is_some()
	}
}

impl State {
	/// Opens Discovery and reads the featured servers, or those matching `query`.
	pub fn search_discovery(&mut self, query: &str) -> Option<Command> {
		let query = query.trim();
		if !query.is_empty() && !valid_query(query) {
			return None;
		}
		let discovery = &mut self.discovery;
		discovery.open = true;
		if discovery
			.pending
			.as_ref()
			.is_some_and(|(pending, _)| pending == query)
		{
			return None;
		}
		discovery.error = None;
		discovery.sequence = discovery.sequence.wrapping_add(1);
		discovery.pending = Some((query.to_owned(), discovery.sequence));
		Some(Command::Discovery {
			query: query.to_owned(),
			request: discovery.sequence,
		})
	}
	pub fn close_discovery(&mut self) {
		self.discovery.open = false;
		self.discovery.error = None;
	}
	pub(crate) fn apply_discovery(&mut self, request: u64, result: Result<Page, Failure>) {
		let discovery = &mut self.discovery;
		let Some((query, _)) = discovery
			.pending
			.take_if(|(_, pending)| *pending == request)
		else {
			return;
		};
		match result {
			Ok(page) if page.query == query && page.valid() => {
				discovery.page = Some(page);
				discovery.error = None;
			}
			Ok(_) => discovery.error = Some(Failure::Protocol),
			Err(failure) => discovery.error = Some(failure),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn only_the_latest_search_applies() {
		let mut state = State::default();
		let page = |query: &str| Page {
			query: query.into(),
			items: Vec::new(),
		};
		let Some(Command::Discovery {
			request: featured, ..
		}) = state.search_discovery("")
		else {
			panic!("opening reads the featured servers");
		};
		assert!(state.search_discovery(" ").is_none());
		let Some(Command::Discovery {
			request: search, ..
		}) = state.search_discovery(" hub ")
		else {
			panic!("a search replaces the featured list");
		};
		state.apply_discovery(featured, Ok(page("")));
		assert!(state.discovery.page.is_none() && state.discovery.loading());
		state.apply_discovery(search, Ok(page("other")));
		assert_eq!(state.discovery.error, Some(Failure::Protocol));
		let Some(Command::Discovery { request, query }) = state.search_discovery("hub") else {
			panic!("searching again reads again");
		};
		assert_eq!(query, "hub");
		state.apply_discovery(request, Ok(page("hub")));
		assert_eq!(state.discovery.page.as_ref().unwrap().query, "hub");
		assert!(state.search_discovery(&"x".repeat(101)).is_none());
	}
}
