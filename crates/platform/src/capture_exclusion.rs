//! Keeps the app window out of screen captures and recordings where the system offers it.
#![allow(unsafe_code)]

/// Hides the window from, or shows it to, screen capture. Platforms without the control, and
/// systems that refuse it, leave the window capturable.
pub fn set_hidden(window: &winit::window::Window, hidden: bool) {
	native::set_hidden(window, hidden);
}

#[cfg(target_os = "windows")]
mod native {
	use windows::Win32::{
		Foundation::HWND,
		UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE},
	};
	use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

	pub fn set_hidden(window: &winit::window::Window, hidden: bool) {
		let Ok(handle) = window.window_handle() else {
			return;
		};
		let RawWindowHandle::Win32(handle) = handle.as_raw() else {
			return;
		};
		let affinity = if hidden {
			WDA_EXCLUDEFROMCAPTURE
		} else {
			WDA_NONE
		};
		// SAFETY: winit owns this HWND for the borrowed window's lifetime.
		let _ = unsafe { SetWindowDisplayAffinity(HWND(handle.hwnd.get() as *mut _), affinity) };
	}
}

#[cfg(target_os = "macos")]
mod native {
	use objc2::MainThreadMarker;
	use objc2_app_kit::{NSView, NSWindowSharingType};
	use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

	pub fn set_hidden(window: &winit::window::Window, hidden: bool) {
		if MainThreadMarker::new().is_none() {
			return;
		}
		let Ok(handle) = window.window_handle() else {
			return;
		};
		let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
			return;
		};
		// SAFETY: winit owns this NSView for the borrowed window's lifetime; AppKit access
		// stays on the main thread.
		let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
		if let Some(window) = view.window() {
			window.setSharingType(if hidden {
				NSWindowSharingType::None
			} else {
				NSWindowSharingType::ReadOnly
			});
		}
	}
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod native {
	pub fn set_hidden(_window: &winit::window::Window, _hidden: bool) {}
}
