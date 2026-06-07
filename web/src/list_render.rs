use std::{borrow::Cow, collections::HashMap, hash::Hash};

use neoview::{PropId, ScopedStoreProv, Store, StoreProv};
use rustc_hash::FxBuildHasher;
use web_sys::Element;

use crate::{chunk::ChunkRemover, context::DomContext, prelude::ChunkBuild};

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
fn render_list_core<T: Clone, TCont: AsRef<[T]>, K: Eq + Hash + 'static>(
	build: &mut ChunkBuild, prop: PropId<TCont>, key_fn: impl Fn(&T) -> K + 'static,
	enumerate: bool, tag: impl Into<Cow<'static, str>>,
	mut item_chunk: impl FnMut(&mut ChunkBuild, T, Option<PropId<usize>>) + 'static,
) {
	let tag = tag.into();
	let len = build.read(prop).as_ref().len();
	let mut old_items = Vec::with_capacity(len);
	let mut old_keys = Vec::with_capacity(len);

	for ind in 0..len {
		let item = build.read(prop).as_ref()[ind].clone();
		old_keys.push(key_fn(&item));
		let item = build_item(build.ctx(), &tag, item, enumerate.then_some(ind), &mut item_chunk);
		build.build_codes.node(item.el.clone().into());
		old_items.push(Some(item))
	}

	let slab = build.slab();
	build.ref_el(move |ctx, parent| {
		let parent = parent.clone();
		let fun = move |ctx: &mut DomContext| {
			let list = ctx.read(prop).as_ref();
			let new_keys: Vec<_> = list.iter().map(|v| key_fn(v)).collect();
			let mut new_items = (0..list.len()).map(|_| None).collect();
			let mut differ = Differ {
				ctx,
				old_items: &mut old_items,
				parent: &parent,
				new_items: &mut new_items,
				item_builder: |ctx, ind| {
					let item = ctx.read(prop).as_ref()[ind].clone();
					build_item(ctx, &tag, item, enumerate.then_some(ind), &mut item_chunk)
				},
			};
			diff(&old_keys, &new_keys, &mut differ);
			(old_items, old_keys) = (new_items, new_keys);
		};
		Store::effect_manual_in(ctx, slab, vec![prop.erase_type()], Vec::new(), fun, false).unwrap()
	});
}

struct Item {
	el: Element,
	index: Option<PropId<usize>>,
	remover: ChunkRemover,
}

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

struct Differ<'a, F: FnMut(&mut DomContext, usize) -> Item> {
	ctx: &'a mut DomContext,
	parent: &'a Element,
	old_items: &'a mut Vec<Option<Item>>,
	new_items: &'a mut Vec<Option<Item>>,
	item_builder: F,
}
impl<F: FnMut(&mut DomContext, usize) -> Item> ReconcileOps for Differ<'_, F> {
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
	fn set_index(&mut self, old_ind: usize, new_ind: usize) {
		let item = self.old_items[old_ind].take().unwrap();
		if let Some(index) = item.index
			&& old_ind != new_ind
		{
			self.ctx.write(index, new_ind);
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
			let ref_idx = (new_end < new_len).then_some(new_end);
			ops.insert(ind, ref_idx);
		}
	} else if start == new_end {
		// Only removals remain.
		for ind in start..old_end {
			ops.remove(ind);
		}
	} else {
		// 4. Map Phase: Build a map of the remaining new items.
		let mut new_ind_map = HashMap::with_capacity_and_hasher(new_end - start, FxBuildHasher);
		for ind in start..new_end {
			new_ind_map.insert(&new[ind], ind);
		}

		let new_left = new_end - start;
		// Tracks where new items came from. `0` means it's a brand new item.
		let mut sources = vec![0isize; new_left];
		let mut moved = false;
		let mut pos = 0;
		let mut patched = 0;

		// Find which old items are kept, moved, or removed.
		for ind in start..old_end {
			if patched >= new_left {
				ops.remove(ind);
				continue;
			}

			if let Some(&new_ind) = new_ind_map.get(&old[ind]) {
				// Item is kept. Record its old index (+1 to reserve 0 for "new").
				sources[new_ind - start] = (ind + 1) as isize;
				ops.set_index(ind, new_ind);

				// If a new index is smaller than a previous one, items crossed paths (moved).
				if new_ind >= pos {
					pos = new_ind;
				} else {
					moved = true;
				}
				patched += 1;
			} else {
				// Item doesn't exist in the new array.
				ops.remove(ind);
			}
		}

		// 5. Patch Phase: Apply DOM mutations backwards.
		// We iterate backwards so the `reference` node is always already in its final position.
		if moved {
			// Find the longest sequence of items that don't need to move (Anchors).
			let seq = longest_increasing_subsequence(&sources);
			let mut j = seq.len() as isize - 1;

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

		// Binary search to find the correct insertion point to replace an existing tail.
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
