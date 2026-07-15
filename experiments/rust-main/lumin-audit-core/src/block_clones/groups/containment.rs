use std::collections::HashMap;

use super::Instance;

struct FileIntervals {
    first_start: usize,
    count: usize,
}

struct RangeQuery {
    start: usize,
    end: usize,
    minimum_end: usize,
}

pub(super) struct ContainmentIndex {
    tree_base: usize,
    max_end_by_node: Vec<usize>,
    intervals_by_start: HashMap<usize, Vec<(usize, usize)>>,
    files: HashMap<String, FileIntervals>,
}

impl ContainmentIndex {
    pub(super) fn new(position_count: usize) -> Self {
        let tree_base = position_count.max(1).next_power_of_two();
        Self {
            tree_base,
            max_end_by_node: vec![0usize; tree_base * 2],
            intervals_by_start: HashMap::new(),
            files: HashMap::new(),
        }
    }

    pub(super) fn insert(&mut self, group_index: usize, instance: &Instance) {
        self.intervals_by_start
            .entry(instance.start_token)
            .or_default()
            .push((instance.end_token, group_index));
        self.files
            .entry(instance.file.clone())
            .and_modify(|file| {
                file.first_start = file.first_start.min(instance.start_token);
                file.count += 1;
            })
            .or_insert(FileIntervals {
                first_start: instance.start_token,
                count: 1,
            });

        let mut node = self.tree_base + instance.start_token;
        self.max_end_by_node[node] = self.max_end_by_node[node].max(instance.end_token);
        node /= 2;
        while node > 0 {
            self.max_end_by_node[node] =
                self.max_end_by_node[node * 2].max(self.max_end_by_node[node * 2 + 1]);
            node /= 2;
        }
    }

    pub(super) fn contains_group(
        &self,
        instances: &[Instance],
        scratch: &mut ContainmentScratch,
    ) -> bool {
        if instances.is_empty() {
            return false;
        }
        scratch.prepare(instances.len());
        scratch.order.extend(0..instances.len());
        scratch.order.sort_by_key(|&index| {
            self.files
                .get(&instances[index].file)
                .map_or(0, |file| file.count)
        });

        let mut first = true;
        for &instance_index in &scratch.order {
            scratch.current.fill(0);
            self.fill_containing_groups(&instances[instance_index], &mut scratch.current);
            if first {
                scratch.possible.copy_from_slice(&scratch.current);
                first = false;
            } else {
                for (possible, current) in scratch.possible.iter_mut().zip(&scratch.current) {
                    *possible &= current;
                }
            }
            if scratch.possible.iter().all(|bits| *bits == 0) {
                return false;
            }
        }
        scratch.possible.iter().any(|bits| *bits != 0)
    }

    fn fill_containing_groups(&self, instance: &Instance, groups: &mut [u64]) {
        let Some(file) = self.files.get(&instance.file) else {
            return;
        };
        if file.first_start > instance.start_token {
            return;
        }
        let query = RangeQuery {
            start: file.first_start,
            end: instance.start_token + 1,
            minimum_end: instance.end_token,
        };
        self.collect_containing_groups(1, 0, self.tree_base, &query, groups);
    }

    fn collect_containing_groups(
        &self,
        node: usize,
        node_start: usize,
        node_end: usize,
        query: &RangeQuery,
        groups: &mut [u64],
    ) {
        if node_end <= query.start
            || node_start >= query.end
            || self.max_end_by_node[node] < query.minimum_end
        {
            return;
        }
        if node_end - node_start == 1 {
            if let Some(intervals) = self.intervals_by_start.get(&node_start) {
                for &(end, group_index) in intervals {
                    if end >= query.minimum_end {
                        groups[group_index / 64] |= 1u64 << (group_index % 64);
                    }
                }
            }
            return;
        }
        let middle = node_start + (node_end - node_start) / 2;
        self.collect_containing_groups(node * 2, node_start, middle, query, groups);
        self.collect_containing_groups(node * 2 + 1, middle, node_end, query, groups);
    }
}

pub(super) struct ContainmentScratch {
    possible: Vec<u64>,
    current: Vec<u64>,
    order: Vec<usize>,
}

impl ContainmentScratch {
    pub(super) fn new(max_groups: usize) -> Self {
        let word_count = max_groups.div_ceil(64);
        Self {
            possible: vec![0u64; word_count],
            current: vec![0u64; word_count],
            order: Vec::new(),
        }
    }

    fn prepare(&mut self, instance_count: usize) {
        self.possible.fill(0);
        self.current.fill(0);
        self.order.clear();
        self.order.reserve(instance_count);
    }
}
