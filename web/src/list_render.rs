//! list rendering through [`render_list`]
use std::{
	borrow::Cow,
	cell::{RefCell, RefMut},
	collections::HashMap,
	hash::Hash,
	rc::Rc,
};

use neoview::{EffectDeps::Manual, PropId, ScopedStoreProv, Store, StoreProv};
use rustc_hash::FxBuildHasher;
use web_sys::Element;

use crate::{chunk::ChunkRemover, context::DomContext, prelude::ChunkBuild};

/// dynamic list rendering primitive.
///
/// the `render_list` function take a reactive property containing a list and construct each item into the curent element.
///
/// when the list changes, `render_list` patch the ui by constructing the new items, removing the old ones and moving the required ones.
///
/// the list is any type impliminting [`AsRef<[T]>`](AsRef), and a `key_fn` that returns a unique key for each item is requried since diff is based on the simple keys for performance reasons.
///
/// an item ui is constructed by creating a `tag` element then targeting it with a [`ChunkBuild`] having its own scope and passing the build with the item to the `item_chunk` function, after that builing the build and inserting the element into the ui.
///
/// # example
/// ```
/// #[derive(Clone)]
/// struct User { id: u32, name: String, age: u32 }
/// let users = read_users(build);
/// render_list(build, users, |v| v.id, "div", |mut build, user| {
///	    chunk!(build, "user ", user.id, ": name = ", user.name, ", age = ", user.age);
/// });
/// ```
///
/// # Notes
/// note that keys must be unique, the renderer handle deplicate keys fully but the state of the ui will be unpredictable, and multiple items having the same key may have different fields leading to chous.
///
/// all item chunks will be removed when the parent chunk is removed.
///
/// for dynamic indexes, see [`render_list_enumerated`].
pub fn render_list<T: Clone, K: Eq + Hash + 'static>(
	build: &mut ChunkBuild, prop: PropId<impl AsRef<[T]>>, key_fn: impl Fn(&T) -> K + 'static,
	tag: impl Into<Cow<'static, str>>, mut item_chunk: impl FnMut(&mut ChunkBuild, T) + 'static,
) {
	#[rustfmt::skip]
	render_list_core(
		build, prop, key_fn, false, tag,
		move |build, item, _| item_chunk(build, item),
	);
}

/// like [`render_list`] but with dynamic indexes.
///
/// the `render_list_enumerated` is exactly like the normal [`render_list`] except additional reactive property reflecting the index of the item in the list is passed to the `item_chunk`.
///
/// # example
/// ```
/// #[derive(Clone)]
/// struct User { id: u32, name: String, age: u32 }
/// let users = read_users(build);
/// render_list_enumerated(build, users, |v| v.id, "div", |mut build, user, index| {
///	   chunk!(build, index, "- user ", user.id, ": name = ", user.name, ", age = ", user.age);
/// });
/// ```
pub fn render_list_enumerated<T: Clone, K: Eq + Hash + 'static>(
	build: &mut ChunkBuild, prop: PropId<impl AsRef<[T]>>, key_fn: impl Fn(&T) -> K + 'static,
	tag: impl Into<Cow<'static, str>>,
	mut item_chunk: impl FnMut(&mut ChunkBuild, T, PropId<usize>) + 'static,
) {
	#[rustfmt::skip]
    render_list_core(
        build, prop, key_fn, true, tag,
        move |build, item, index| item_chunk(build, item, index.unwrap()),
    );
}

/// the implementation of [`render_list`] and [`render_list_enumerated`].
///
/// `enumated` is a flag for giving the `item_chunk` the index of the item.
fn render_list_core<T: Clone, K: Eq + Hash + 'static>(
	build: &mut ChunkBuild, prop: PropId<impl AsRef<[T]>>, key_fn: impl Fn(&T) -> K + 'static,
	enumerate: bool, tag: impl Into<Cow<'static, str>>,
	mut item_chunk: impl FnMut(&mut ChunkBuild, T, Option<PropId<usize>>) + 'static,
) {
	let tag = tag.into();
	let len = build.read(prop).as_ref().len();
	// stored inside an `Option` for taking without move.
	let mut items = Vec::with_capacity(len);
	let mut keys = Vec::with_capacity(len);

	// init render
	for ind in 0..len {
		let item = build.read(prop).as_ref()[ind].clone();
		keys.push(key_fn(&item));

		let item = build_item(build.ctx(), &tag, item, enumerate.then_some(ind), &mut item_chunk);
		build.build_codes.node(item.el.clone().into());

		items.push(Some(item))
	}

	// it is used by the patcher effect and the remover
	let items = Rc::new(RefCell::new(items));
	let items_clone = items.clone();

	let slab = build.slab();
	build.ref_el(move |ctx, parent| {
		let parent = parent.clone();

		let patcher = move |ctx: &mut DomContext| {
			let list = ctx.read(prop).as_ref();
			let new_keys: Vec<_> = list.iter().map(|v| key_fn(v)).collect();
			let mut new_items = (0..list.len()).map(|_| None).collect();

			let mut patcher = Patcher {
				ctx,
				old_items: items.borrow_mut(),
				parent: &parent,
				new_items: &mut new_items,
				item_builder: |ctx, ind| {
					let item = ctx.read(prop).as_ref()[ind].clone();
					build_item(ctx, &tag, item, enumerate.then_some(ind), &mut item_chunk)
				},
			};
			diff(&keys, &new_keys, &mut patcher);

			drop(patcher);
			(*items.borrow_mut(), keys) = (new_items, new_keys);
		};
		// run the patcher after the chunk is built since the list may had been changed
		let ty = Manual { read: vec![prop.erase_type()], write: Vec::new(), init_run: true };
		Store::effect(ctx, slab, ty, patcher).unwrap();

		let remover = move |ctx: &mut DomContext| {
			for item in items_clone.borrow_mut().drain(..) {
				item.unwrap().remover.remove(ctx);
			}
		};
		ctx.store().add_cleaner(slab, remover).unwrap()
	});
}

/// an item in the list
struct Item {
	/// its element
	el: Element,
	/// the index prop if needed
	index: Option<PropId<usize>>,
	/// the item chunk remover
	remover: ChunkRemover,
}

/// construct the ui of an item
fn build_item<T>(
	ctx: &mut DomContext, tag: &str, item: T, ind: Option<usize>,
	item_chunk: &mut impl FnMut(&mut ChunkBuild, T, Option<PropId<usize>>),
) -> Item {
	let mut chunk = ctx.removable_chunk(&tag);
	let index = ind.map(|ind| chunk.prop(ind));
	item_chunk(&mut chunk, item, index);
	let (el, remover) = chunk.build();
	Item { el, index, remover }
}

/// patch the list ui
struct Patcher<'a, F: FnMut(&mut DomContext, usize) -> Item> {
	ctx: &'a mut DomContext,
	parent: &'a Element,
	old_items: RefMut<'a, Vec<Option<Item>>>,
	new_items: &'a mut Vec<Option<Item>>,
	item_builder: F,
}
impl<F: FnMut(&mut DomContext, usize) -> Item> ReconcileOps for Patcher<'_, F> {
	fn insert(&mut self, new_ind: usize, reference: Option<usize>) {
		let item = (self.item_builder)(self.ctx, new_ind);

		match reference {
			Some(before) => {
				let el = &self.new_items[before].as_ref().unwrap().el;
				el.before_with_node_1(&item.el).unwrap()
			}
			None => self.parent.append_with_node_1(&item.el).unwrap(),
		}

		self.new_items[new_ind] = Some(item);
	}
	fn _move(&mut self, new_ind: usize, reference: Option<usize>) {
		let item = self.new_items[new_ind].as_ref().unwrap();
		match reference {
			Some(before) => {
				self.new_items[before].as_ref().unwrap().el.before_with_node_1(&item.el)
			}
			None => self.parent.append_with_node_1(&item.el),
		}
		.unwrap();
	}
	fn remove(&mut self, old_ind: usize) {
		self.old_items[old_ind].take().unwrap().remover.remove(self.ctx);
	}
	/// update the index and move to new_items
	fn set_index(&mut self, old_ind: usize, new_ind: usize) {
		let item = self.old_items[old_ind].take().unwrap();
		if let Some(index) = item.index
			&& old_ind != new_ind
		{
			self.ctx.write(index, new_ind);
			// we are in an effect
			self.ctx.store().force_update(index);
		}
		self.new_items[new_ind] = Some(item);
	}
}

pub trait ReconcileOps {
	/// Inserts an item before `reference` item.
	/// If `reference` is None, appends to the end of the container.
	fn insert(&mut self, new_ind: usize, reference: Option<usize>);

	/// move an item before `reference` item.
	/// If `reference` is None, appends to the end of the container.
	fn _move(&mut self, new_ind: usize, reference: Option<usize>);

	/// Removes an item.
	fn remove(&mut self, old_ind: usize);

	/// Fires once for every kept node to update its index prop.
	fn set_index(&mut self, old_ind: usize, new_ind: usize);
}

/// compute the diff between two lists
pub fn diff<T: Eq + Hash, O: ReconcileOps>(old: &[T], new: &[T], ops: &mut O) {
	let old_len = old.len();
	let new_len = new.len();

	// 1. Common Prefix: Skip matching items at the start.
	let mut start = 0;
	while start < old_len && start < new_len && old[start] == new[start] {
		ops.set_index(start, start);
		start += 1;
	}

	// 2. Common Suffix: Skip matching items at the end.
	let mut old_end = old_len;
	let mut new_end = new_len;
	while old_end > start && new_end > start && old[old_end - 1] == new[new_end - 1] {
		old_end -= 1;
		new_end -= 1;
		ops.set_index(old_end, new_end);
	}

	// 3. Fast Paths: If we only have insertions or removals left.
	if start == old_end {
		// Only insertions remain.
		for ind in start..new_end {
			let ref_ind = (new_end < new_len).then_some(new_end);
			ops.insert(ind, ref_ind);
		}
	} else if start == new_end {
		// Only removals remain.
		for ind in start..old_end {
			ops.remove(ind);
		}
	} else {
		// 4. Map Phase: Build a map of the remaining new items.
		let new_left = new_end - start;
		// T -> new ind
		let mut new_ind_map = HashMap::with_capacity_and_hasher(new_left, FxBuildHasher);
		let mut next_duplicate = vec![None; new_left];

		for ind in (start..new_end).rev() {
			if let Some(&existing_ind) = new_ind_map.get(&new[ind]) {
				next_duplicate[ind - start] = Some(existing_ind);
			}
			new_ind_map.insert(&new[ind], ind);
		}

		// Tracks where new items came from. `0` means it's a brand new item.
		let mut sources = vec![0isize; new_left];
		let mut some_moved = false;
		let mut pos = 0;
		// without removed
		let mut items_patched = 0;

		// Find which old items are kept, moved, or removed.
		for ind in start..old_end {
			// no more new items, remove
			if items_patched >= new_left {
				ops.remove(ind);
				continue;
			}

			if let Some(new_ind) = new_ind_map.remove(&old[ind]) {
				let source_ind = new_ind - start;

				if let Some(next_ind) = next_duplicate[source_ind] {
					new_ind_map.insert(&new[next_ind], next_ind);
				}

				// Item is kept. Record its old index (+1 to reserve 0 for "new").
				sources[source_ind] = (ind + 1) as isize;
				ops.set_index(ind, new_ind);

				// If a new index is smaller than a previous one, items crossed paths (moved).
				if new_ind >= pos {
					pos = new_ind;
				} else {
					some_moved = true;
				}
				items_patched += 1;
				continue;
			}

			// Item doesn't exist in the new array
			ops.remove(ind);
		}

		// 5. Patch Phase: Apply DOM mutations backwards.
		if some_moved {
			// Find the longest sequence of items that don't need to move.
			let seq = longest_increasing_subsequence(&sources);
			let mut j = seq.len() as isize - 1;

			// in reverse to make `reference` always in its final position
			for ind in (0..new_left).rev() {
				let new_ind = start + ind;
				let ref_ind = (new_ind + 1 < new_len).then_some(new_ind + 1);

				if sources[ind] == 0 {
					// Brand new item.
					ops.insert(new_ind, ref_ind);
				} else if j < 0 || ind != seq[j as usize] {
					// Item exists, but isn't an anchor. MOVE it.
					ops._move(new_ind, ref_ind);
				} else {
					// Item is an anchor. Leave it exactly where it is.
					j -= 1;
				}
			}
		} else {
			// Optimization: Nothing moved, just insert the new items in the gaps.
			for ind in (0..new_left).rev() {
				if sources[ind] == 0 {
					let new_ind = start + ind;
					let ref_ind = (new_ind + 1 < new_len).then_some(new_ind + 1);
					ops.insert(new_ind, ref_ind);
				}
			}
		}
	}
}

/// Calculates the Longest Increasing Subsequence in O(N log N) using Patience Sorting.
/// Returns the indices of the elements that form the sequence.
fn longest_increasing_subsequence(a: &[isize]) -> Vec<usize> {
	let mut pred = vec![0; a.len()]; // Predecessor tracking for backtracking.
	let mut result = Vec::new(); // Stores indices of the smallest tails seen so far.

	for ind in 0..a.len() {
		if a[ind] == 0 {
			continue;
		} // Ignore completely new nodes.

		let j = result.len();
		if j == 0 || a[result[j - 1]] < a[ind] {
			// Found a larger item. Extend the subsequence.
			if j > 0 {
				pred[ind] = result[j - 1];
			}
			result.push(ind);
			continue;
		}

		let pos = result.partition_point(|&idx| a[idx] < a[ind]);

		if pos > 0 {
			pred[ind] = result[pos - 1];
		}
		result[pos] = ind;
	}

	// Backtrack through predecessors to build the exact anchor sequence.
	let mut u = result.len();
	let mut seq = vec![0; u];
	let mut v = if u > 0 { result[u - 1] } else { 0 };

	while u > 0 {
		u -= 1;
		seq[u] = v;
		v = pred[v];
	}

	seq
}
