// INFO: Computes line-level Git diff indicators (added, modified, deleted) between baseline and working buffer.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineGitChange {
    Added,
    Modified,
    Deleted,
}

impl LineGitChange {
    pub fn color(&self) -> [f32; 4] {
        match self {
            Self::Added => [0.45, 0.78, 0.45, 0.95],    // Subtle green
            Self::Modified => [0.88, 0.68, 0.28, 0.95], // Subtle amber / gold
            Self::Deleted => [0.85, 0.30, 0.30, 0.95],  // Subtle red
        }
    }
}

pub const GIT_DELETED_COLOR: [f32; 4] = [0.85, 0.30, 0.30, 0.95]; // Subtle red

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineGitStatus {
    pub change: Option<LineGitChange>,
    pub deleted_above: bool,
    pub deleted_below: bool,
    pub deleted_lines: Vec<String>,
    pub original_line: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditOp {
    Keep(usize, usize),
    Delete(usize),
    Insert(usize),
}

pub fn myers_diff(orig: &[&str], curr: &[&str]) -> Vec<EditOp> {
    let n = orig.len();
    let m = curr.len();
    if n == 0 && m == 0 {
        return Vec::new();
    }
    if n == 0 {
        return (0..m).map(EditOp::Insert).collect();
    }
    if m == 0 {
        return (0..n).map(EditOp::Delete).collect();
    }

    // 1. Fast common prefix trimming
    let mut prefix_len = 0;
    while prefix_len < n && prefix_len < m && orig[prefix_len] == curr[prefix_len] {
        prefix_len += 1;
    }

    // 2. Fast common suffix trimming
    let mut suffix_len = 0;
    while suffix_len < (n - prefix_len)
        && suffix_len < (m - prefix_len)
        && orig[n - 1 - suffix_len] == curr[m - 1 - suffix_len]
    {
        suffix_len += 1;
    }

    // If whole sequences match completely
    if prefix_len + suffix_len == n && prefix_len + suffix_len == m {
        return (0..n).map(|i| EditOp::Keep(i, i)).collect();
    }

    let mut ops = Vec::with_capacity(n + m);
    for i in 0..prefix_len {
        ops.push(EditOp::Keep(i, i));
    }

    let mid_orig_len = n - prefix_len - suffix_len;
    let mid_curr_len = m - prefix_len - suffix_len;
    let mid_orig = &orig[prefix_len..n - suffix_len];
    let mid_curr = &curr[prefix_len..m - suffix_len];

    if mid_orig_len == 0 {
        for j in 0..mid_curr_len {
            ops.push(EditOp::Insert(prefix_len + j));
        }
    } else if mid_curr_len == 0 {
        for i in 0..mid_orig_len {
            ops.push(EditOp::Delete(prefix_len + i));
        }
    } else {
        // Run bounded core diff on trimmed middle
        let mid_ops = core_diff(mid_orig, mid_curr);
        for op in mid_ops {
            match op {
                EditOp::Keep(o, c) => ops.push(EditOp::Keep(prefix_len + o, prefix_len + c)),
                EditOp::Delete(o) => ops.push(EditOp::Delete(prefix_len + o)),
                EditOp::Insert(c) => ops.push(EditOp::Insert(prefix_len + c)),
            }
        }
    }

    for s in 0..suffix_len {
        ops.push(EditOp::Keep(n - suffix_len + s, m - suffix_len + s));
    }

    ops
}

fn core_diff(orig: &[&str], curr: &[&str]) -> Vec<EditOp> {
    let n = orig.len();
    let m = curr.len();
    let max = n + m;

    // For extremely large middle blocks, partition on unique matching anchor lines
    if max > 3000 {
        if let Some((anchor_orig, anchor_curr)) = find_unique_anchor(orig, curr) {
            let mut left_ops = myers_diff(&orig[..anchor_orig], &curr[..anchor_curr]);
            left_ops.push(EditOp::Keep(anchor_orig, anchor_curr));
            let right_ops = myers_diff(&orig[anchor_orig + 1..], &curr[anchor_curr + 1..]);
            for op in right_ops {
                match op {
                    EditOp::Keep(o, c) => {
                        left_ops.push(EditOp::Keep(anchor_orig + 1 + o, anchor_curr + 1 + c));
                    }
                    EditOp::Delete(o) => left_ops.push(EditOp::Delete(anchor_orig + 1 + o)),
                    EditOp::Insert(c) => left_ops.push(EditOp::Insert(anchor_curr + 1 + c)),
                }
            }
            return left_ops;
        }

        // Fallback for massive wholesale changes without anchors
        let mut fallback = Vec::with_capacity(max);
        for i in 0..n {
            fallback.push(EditOp::Delete(i));
        }
        for j in 0..m {
            fallback.push(EditOp::Insert(j));
        }
        return fallback;
    }

    let max_d = max.min(800);
    let offset = max as isize;
    let mut v = vec![0isize; 2 * max + 1];
    let mut trace: Vec<Vec<isize>> = Vec::with_capacity(max_d + 1);

    let mut final_d = 0;
    let mut reached = false;

    for d in 0..=max_d {
        trace.push(v.clone());
        let d_isize = d as isize;
        for k in (-d_isize..=d_isize).step_by(2) {
            let k_idx = (k + offset) as usize;
            let mut x = if k == -d_isize
                || (k != d_isize && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize])
            {
                v[(k + 1 + offset) as usize]
            } else {
                v[(k - 1 + offset) as usize] + 1
            };
            let mut y = x - k;

            while (x as usize) < n && (y as usize) < m && orig[x as usize] == curr[y as usize] {
                x += 1;
                y += 1;
            }

            v[k_idx] = x;

            if (x as usize) >= n && (y as usize) >= m {
                final_d = d;
                reached = true;
                break;
            }
        }
        if reached {
            break;
        }
    }

    if !reached {
        let mut fallback = Vec::with_capacity(max);
        for i in 0..n {
            fallback.push(EditOp::Delete(i));
        }
        for j in 0..m {
            fallback.push(EditOp::Insert(j));
        }
        return fallback;
    }

    // Backtrack to extract the edit path
    let mut ops = Vec::new();
    let mut x = n as isize;
    let mut y = m as isize;

    for d in (1..=final_d).rev() {
        let v_prev = &trace[d];
        let k = x - y;
        let d_isize = d as isize;
        let prev_k = if k == -d_isize
            || (k != d_isize
                && v_prev[(k - 1 + offset) as usize] < v_prev[(k + 1 + offset) as usize])
        {
            k + 1
        } else {
            k - 1
        };

        let prev_x = v_prev[(prev_k + offset) as usize];
        let prev_y = prev_x - prev_k;

        while x > prev_x && y > prev_y {
            x -= 1;
            y -= 1;
            ops.push(EditOp::Keep(x as usize, y as usize));
        }

        if x == prev_x {
            y -= 1;
            ops.push(EditOp::Insert(y as usize));
        } else {
            x -= 1;
            ops.push(EditOp::Delete(x as usize));
        }
    }

    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        ops.push(EditOp::Keep(x as usize, y as usize));
    }

    ops.reverse();
    ops
}

fn find_unique_anchor(orig: &[&str], curr: &[&str]) -> Option<(usize, usize)> {
    use std::collections::HashMap;
    let mut orig_counts: HashMap<&str, (usize, usize)> = HashMap::new(); // (count, index)
    for (i, line) in orig.iter().enumerate() {
        let entry = orig_counts.entry(line).or_insert((0, i));
        entry.0 += 1;
    }

    let mut curr_counts: HashMap<&str, (usize, usize)> = HashMap::new();
    for (j, line) in curr.iter().enumerate() {
        let entry = curr_counts.entry(line).or_insert((0, j));
        entry.0 += 1;
    }

    // Find the best unique line near the middle
    let mid = orig.len() / 2;
    let mut best_anchor: Option<(usize, usize)> = None;
    let mut min_dist = usize::MAX;

    for (&line, &(o_count, o_idx)) in &orig_counts {
        if o_count == 1
            && let Some(&(c_count, c_idx)) = curr_counts.get(line)
            && c_count == 1
        {
            let dist = (o_idx as isize - mid as isize).unsigned_abs();
            if dist < min_dist {
                min_dist = dist;
                best_anchor = Some((o_idx, c_idx));
            }
        }
    }

    best_anchor
}

pub fn compute_line_diff(orig_lines: &[&str], curr_lines: &[&str]) -> Vec<LineGitStatus> {
    let mut statuses = vec![LineGitStatus::default(); curr_lines.len()];
    if curr_lines.is_empty() {
        return statuses;
    }

    if orig_lines.is_empty() {
        for st in &mut statuses {
            st.change = Some(LineGitChange::Added);
        }
        return statuses;
    }

    let ops = myers_diff(orig_lines, curr_lines);

    let mut i = 0;
    while i < ops.len() {
        match ops[i] {
            EditOp::Keep(_, _) => {
                i += 1;
            }
            EditOp::Delete(_) | EditOp::Insert(_) => {
                let mut deletions = 0;
                let mut deleted_text = Vec::new();
                let mut insertions = Vec::new();
                while i < ops.len() {
                    match ops[i] {
                        EditOp::Delete(orig_idx) => {
                            deletions += 1;
                            if orig_idx < orig_lines.len() {
                                deleted_text.push(orig_lines[orig_idx].to_string());
                            }
                            i += 1;
                        }
                        EditOp::Insert(b_idx) => {
                            insertions.push(b_idx);
                            i += 1;
                        }
                        EditOp::Keep(_, _) => break,
                    }
                }

                let num_ins = insertions.len();
                if deletions > 0 && num_ins > 0 {
                    let num_mod = deletions.min(num_ins);
                    for (idx, &b_idx) in insertions[..num_mod].iter().enumerate() {
                        if b_idx < statuses.len() {
                            statuses[b_idx].change = Some(LineGitChange::Modified);
                            if idx < deleted_text.len() {
                                statuses[b_idx].original_line = Some(deleted_text[idx].clone());
                            }
                        }
                    }
                    for &b_idx in &insertions[num_mod..] {
                        if b_idx < statuses.len() {
                            statuses[b_idx].change = Some(LineGitChange::Added);
                        }
                    }
                    if deletions > num_ins
                        && let Some(&first_ins) = insertions.first()
                        && first_ins < statuses.len()
                    {
                        statuses[first_ins].deleted_above = true;
                        statuses[first_ins].deleted_lines = deleted_text[num_mod..].to_vec();
                    }
                } else if num_ins > 0 {
                    for &b_idx in &insertions {
                        if b_idx < statuses.len() {
                            statuses[b_idx].change = Some(LineGitChange::Added);
                        }
                    }
                } else if deletions > 0 {
                    let next_b = if i < ops.len() {
                        if let EditOp::Keep(_, b) = ops[i] {
                            Some(b)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(b) = next_b {
                        if b < statuses.len() {
                            statuses[b].deleted_above = true;
                            statuses[b].deleted_lines = deleted_text;
                        }
                    } else if !statuses.is_empty() {
                        let last = statuses.len() - 1;
                        statuses[last].deleted_below = true;
                        statuses[last].deleted_lines = deleted_text;
                    }
                }
            }
        }
    }

    statuses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_lines_have_no_changes() {
        let orig = &["line 1", "line 2", "line 3"];
        let curr = &["line 1", "line 2", "line 3"];
        let diff = compute_line_diff(orig, curr);
        assert_eq!(diff.len(), 3);
        assert!(
            diff.iter()
                .all(|s| s.change.is_none() && !s.deleted_above && !s.deleted_below)
        );
    }

    #[test]
    fn test_added_line() {
        let orig = &["line 1", "line 3"];
        let curr = &["line 1", "line 2", "line 3"];
        let diff = compute_line_diff(orig, curr);
        assert_eq!(diff.len(), 3);
        assert_eq!(diff[0].change, None);
        assert_eq!(diff[1].change, Some(LineGitChange::Added));
        assert_eq!(diff[2].change, None);
    }

    #[test]
    fn test_modified_line() {
        let orig = &["line 1", "old line", "line 3"];
        let curr = &["line 1", "new line", "line 3"];
        let diff = compute_line_diff(orig, curr);
        assert_eq!(diff.len(), 3);
        assert_eq!(diff[0].change, None);
        assert_eq!(diff[1].change, Some(LineGitChange::Modified));
        assert_eq!(diff[1].original_line.as_deref(), Some("old line"));
        assert_eq!(diff[2].change, None);
    }

    #[test]
    fn test_deleted_line_middle() {
        let orig = &["line 1", "to delete", "line 3"];
        let curr = &["line 1", "line 3"];
        let diff = compute_line_diff(orig, curr);
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].change, None);
        assert!(!diff[0].deleted_above);
        assert_eq!(diff[1].change, None);
        assert!(diff[1].deleted_above);
        assert_eq!(diff[1].deleted_lines, vec!["to delete".to_string()]);
    }

    #[test]
    fn test_deleted_line_at_end() {
        let orig = &["line 1", "line 2", "to delete"];
        let curr = &["line 1", "line 2"];
        let diff = compute_line_diff(orig, curr);
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].change, None);
        assert_eq!(diff[1].change, None);
        assert!(diff[1].deleted_below);
        assert_eq!(diff[1].deleted_lines, vec!["to delete".to_string()]);
    }

    #[test]
    fn test_empty_orig_all_added() {
        let orig = &[];
        let curr = &["new 1", "new 2"];
        let diff = compute_line_diff(orig, curr);
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].change, Some(LineGitChange::Added));
        assert_eq!(diff[1].change, Some(LineGitChange::Added));
    }

    #[test]
    fn test_large_diff_45000_added_lines() {
        let orig = &["header", "footer"];
        let mut curr_vec = Vec::with_capacity(50_000);
        curr_vec.push("header");
        curr_vec.extend(std::iter::repeat_n("added_line", 48_000));
        curr_vec.push("footer");

        let start = std::time::Instant::now();
        let diff = compute_line_diff(orig, &curr_vec);
        let elapsed = start.elapsed();

        assert_eq!(diff.len(), 48_002);
        assert_eq!(diff[0].change, None);
        assert_eq!(diff[1].change, Some(LineGitChange::Added));
        assert_eq!(diff[48000].change, Some(LineGitChange::Added));
        assert_eq!(diff[48001].change, None);
        assert!(
            elapsed.as_millis() < 500,
            "48,000 added lines must compute in under 500ms, took {:?}",
            elapsed
        );
    }

    #[test]
    fn test_large_diff_10000_mixed_lines() {
        let mut orig_vec = Vec::with_capacity(10_000);
        let mut curr_vec = Vec::with_capacity(10_000);
        for i in 0..10_000 {
            if i % 10 == 0 {
                orig_vec.push("old_unique_line");
                curr_vec.push("new_unique_line");
            } else {
                orig_vec.push("common_line");
                curr_vec.push("common_line");
            }
        }

        let start = std::time::Instant::now();
        let diff = compute_line_diff(&orig_vec, &curr_vec);
        let elapsed = start.elapsed();

        assert_eq!(diff.len(), 10_000);
        assert!(
            elapsed.as_millis() < 500,
            "10,000 mixed lines must compute in under 500ms, took {:?}",
            elapsed
        );
    }
}
