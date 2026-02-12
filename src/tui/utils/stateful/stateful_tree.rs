use tui_tree_widget::{TreeItem, TreeState};

/// Represents which request is currently selected in the tree.
/// This maps tree selection paths back to collection/folder/request indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedRequest {
	/// A root-level request: (collection_index, request_index)
	RootRequest(usize, usize),
	/// A request inside a folder: (collection_index, folder_index, request_index)
	FolderRequest(usize, usize, usize),
}

impl SelectedRequest {
	/// Returns the collection index.
	pub fn collection_index(&self) -> usize {
		match self {
			SelectedRequest::RootRequest(c, _) => *c,
			SelectedRequest::FolderRequest(c, _, _) => *c,
		}
	}

	/// Returns the request index (within root requests or within a folder).
	pub fn request_index(&self) -> usize {
		match self {
			SelectedRequest::RootRequest(_, r) => *r,
			SelectedRequest::FolderRequest(_, _, r) => *r,
		}
	}

	/// Returns the folder index, if the request is inside a folder.
	pub fn folder_index(&self) -> Option<usize> {
		match self {
			SelectedRequest::RootRequest(_, _) => None,
			SelectedRequest::FolderRequest(_, f, _) => Some(*f),
		}
	}
}

#[derive(Default)]
pub struct StatefulTree<'a> {
	pub state: TreeState<usize>,
	pub items: Vec<TreeItem<'a, usize>>,
	pub selected: Option<SelectedRequest>,
}

impl StatefulTree<'_> {
	pub fn up(&mut self) {
		self.state.key_up();
	}

	pub fn down(&mut self) {
		self.state.key_down();
	}

	/// Call this when the user selects a request (depth 2 or 3 in the tree).
	/// `folder_count` is the number of folders in the selected collection,
	/// used to determine whether a depth-2 selection is a folder or a root request.
	pub fn set_selected_with_context(&mut self, folder_count: usize) {
		let path = self.state.selected();
		match path.len() {
			2 => {
				let collection_index = path[0];
				let child_index = path[1];
				if child_index < folder_count {
					// This is a folder, not a request - don't select
					self.selected = None;
				} else {
					// Root-level request (offset by folder count)
					self.selected = Some(SelectedRequest::RootRequest(
						collection_index,
						child_index - folder_count,
					));
				}
			}
			3 => {
				let collection_index = path[0];
				let folder_index = path[1];
				let request_index = path[2];
				self.selected = Some(SelectedRequest::FolderRequest(
					collection_index,
					folder_index,
					request_index,
				));
			}
			_ => {
				self.selected = None;
			}
		}
	}

	pub fn set_unselected(&mut self) {
		self.selected = None;
	}
}
