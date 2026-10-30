//! turning a commit list into the lanes of a `git log --graph` style drawing

use std::collections::HashMap;

use crate::features::git::Commit;

/// A line between two lanes, covering the height of one row.
///
/// For an incoming segment `from` is the lane at the top of the row and
/// `to` the lane of the dot. For an outgoing one `from` is the dot's lane
/// and `to` the lane at the bottom of the row. A passing segment has the
/// same `from` and `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub from: usize,
    pub to: usize,
    /// An index into the palette. Callers wrap it, so it can grow freely.
    pub color: usize,
}

/// Everything needed to draw the graph beside one commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRow {
    pub lane: usize,
    pub color: usize,
    pub is_merge: bool,
    pub incoming: Vec<Segment>,
    pub outgoing: Vec<Segment>,
    pub passing: Vec<Segment>,
    /// How many lanes this row touches, for sizing the graph column.
    pub width: usize,
}

#[derive(Debug, Clone)]
struct ActiveLane {
    color: usize,
}

/// The lane state left after laying out some commits, so the graph can
/// carry on when the next page of older commits arrives.
#[derive(Debug, Clone, Default)]
pub struct GraphBuilder {
    lanes: Vec<Option<ActiveLane>>,
    // which lanes are heading toward each commit
    awaiting: HashMap<String, Vec<usize>>,
    next_color: usize,
}

/// takes the lowest free slot so the trunk stays at the left edge
fn claim(lanes: &mut Vec<Option<ActiveLane>>, lane: ActiveLane) -> usize {
    match lanes.iter().position(Option::is_none) {
        Some(free) => {
            lanes[free] = Some(lane);
            free
        },
        None => {
            lanes.push(Some(lane));
            lanes.len() - 1
        },
    }
}

/// Assigns every commit a lane and the line segments around it.
///
/// Expects children before parents, which is what `git log --date-order`
/// produces. A parent missing from the list, because the history was cut
/// off, simply leaves its lane running off the bottom.
// only the tests lay out a whole history in one go; the app pages through it
#[cfg(test)]
pub fn build_graph(commits: &[Commit]) -> Vec<GraphRow> {
    GraphBuilder::default().extend(commits)
}

impl GraphBuilder {
    /// Lays out `commits`, which must be the next older stretch of the
    /// history this builder has seen so far, and returns one row each.
    pub fn extend(&mut self, commits: &[Commit]) -> Vec<GraphRow> {
        let Self {
            lanes,
            awaiting,
            next_color,
        } = self;
        let mut rows = Vec::with_capacity(commits.len());

        for commit in commits {
            let mut converging = awaiting.remove(commit.hash.as_str()).unwrap_or_default();
            converging.sort_unstable();

            // a commit nobody was waiting for is a branch tip
            let (lane, color) = match converging.first() {
                Some(&index) => (index, lanes[index].as_ref().map_or(0, |l| l.color)),
                None => {
                    let color = *next_color;
                    *next_color += 1;
                    // held only until the slots are cleared below
                    let lane = claim(lanes, ActiveLane { color });
                    (lane, color)
                },
            };

            let incoming: Vec<Segment> = converging
                .iter()
                .filter_map(|&i| {
                    lanes[i].as_ref().map(|l| Segment {
                        from: i,
                        to: lane,
                        color: l.color,
                    })
                })
                .collect();

            let passing: Vec<Segment> = lanes
                .iter()
                .enumerate()
                .filter(|(i, _)| !converging.contains(i) && *i != lane)
                .filter_map(|(i, slot)| {
                    slot.as_ref().map(|l| Segment {
                        from: i,
                        to: i,
                        color: l.color,
                    })
                })
                .collect();

            for &i in &converging {
                lanes[i] = None;
            }
            lanes[lane] = None;

            let mut outgoing = Vec::new();
            for (n, parent) in commit.parents.iter().enumerate() {
                // The first parent always carries on in this lane, even when
                // another lane already waits for it. Both lanes then converge
                // on the parent's row, which is how a side branch rejoins.
                let existing = if n == 0 {
                    None
                } else {
                    awaiting.get(parent.as_str()).and_then(|waiting| waiting.iter().min().copied())
                };

                match existing {
                    Some(index) => {
                        let line_color = lanes[index].as_ref().map_or(color, |l| l.color);
                        outgoing.push(Segment {
                            from: lane,
                            to: index,
                            color: line_color,
                        });
                    },
                    None if n == 0 => {
                        lanes[lane] = Some(ActiveLane { color });
                        awaiting.entry(parent.clone()).or_default().push(lane);
                        outgoing.push(Segment {
                            from: lane,
                            to: lane,
                            color,
                        });
                    },
                    None => {
                        let line_color = *next_color;
                        *next_color += 1;
                        let to = claim(lanes, ActiveLane { color: line_color });
                        awaiting.entry(parent.clone()).or_default().push(to);
                        outgoing.push(Segment {
                            from: lane,
                            to,
                            color: line_color,
                        });
                    },
                }
            }

            // a root commit, or a first parent that merged into another lane,
            // can leave empty slots on the right
            while matches!(lanes.last(), Some(None)) {
                lanes.pop();
            }

            let width = incoming
                .iter()
                .chain(&outgoing)
                .chain(&passing)
                .flat_map(|s| [s.from, s.to])
                .chain(std::iter::once(lane))
                .max()
                .map_or(1, |m| m + 1);

            rows.push(GraphRow {
                lane,
                color,
                is_merge: commit.parents.len() > 1,
                incoming,
                outgoing,
                passing,
                width,
            });
        }

        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(hash: &str, parents: &[&str]) -> Commit {
        Commit {
            hash: hash.to_string(),
            parents: parents.iter().map(|p| p.to_string()).collect(),
            subject: String::new(),
            refs: Vec::new(),
            author: String::new(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_build_graph_empty() {
        assert!(build_graph(&[]).is_empty());
    }

    #[test]
    fn test_build_graph_linear_history_stays_in_lane_zero() {
        let rows = build_graph(&[commit("c", &["b"]), commit("b", &["a"]), commit("a", &[])]);
        assert!(rows.iter().all(|r| r.lane == 0 && r.width == 1));
        assert!(rows[0].incoming.is_empty());
        assert_eq!(rows[1].incoming.len(), 1);
        assert!(rows[2].outgoing.is_empty());
        let colors: Vec<usize> = rows.iter().map(|r| r.color).collect();
        assert_eq!(colors, vec![0, 0, 0]);
    }

    #[test]
    fn test_build_graph_merge_forks_into_second_lane() {
        // m merges side (s) into main (b), both descending from a
        let rows = build_graph(&[
            commit("m", &["b", "s"]),
            commit("s", &["a"]),
            commit("b", &["a"]),
            commit("a", &[]),
        ]);
        assert!(rows[0].is_merge);
        assert_eq!(rows[0].outgoing.len(), 2);
        assert_eq!(rows[0].outgoing[1].to, 1);
        assert_eq!(rows[1].lane, 1);
        assert_ne!(rows[1].color, rows[0].color);
        // lane 0 runs behind the side commit
        assert_eq!(rows[1].passing.len(), 1);
        assert_eq!(rows[1].passing[0].from, 0);
    }

    #[test]
    fn test_build_graph_side_branch_rejoins_trunk() {
        let rows = build_graph(&[
            commit("m", &["b", "s"]),
            commit("s", &["a"]),
            commit("b", &["a"]),
            commit("a", &[]),
        ]);
        // both lanes are still open when a is reached and fold into it
        assert_eq!(
            rows[1].outgoing,
            vec![Segment {
                from: 1,
                to: 1,
                color: rows[1].color
            }]
        );
        assert_eq!(rows[3].lane, 0);
        assert_eq!(rows[3].incoming.len(), 2);
        assert_eq!(
            rows[3].incoming[1],
            Segment {
                from: 1,
                to: 0,
                color: rows[1].color
            }
        );
    }

    #[test]
    fn test_build_graph_converging_lanes_all_drawn_into_dot() {
        // two tips share a parent
        let rows = build_graph(&[commit("x", &["a"]), commit("y", &["a"]), commit("a", &[])]);
        assert_eq!(rows[1].lane, 1);
        assert_eq!(rows[2].incoming.len(), 2);
        assert_eq!(rows[2].lane, 0);
    }

    #[test]
    fn test_build_graph_lane_is_reused_after_closing() {
        let rows = build_graph(&[
            commit("t1", &["a"]),
            commit("t2", &["b"]),
            commit("a", &[]),
            commit("t3", &["c"]),
            commit("b", &[]),
            commit("c", &[]),
        ]);
        // t2 opened lane 1, a closed lane 0, so t3 takes lane 0 again
        assert_eq!(rows[1].lane, 1);
        assert_eq!(rows[3].lane, 0);
    }

    #[test]
    fn test_build_graph_missing_parent_keeps_lane_open() {
        let rows = build_graph(&[commit("c", &["gone"]), commit("d", &[])]);
        assert_eq!(rows[0].outgoing.len(), 1);
        assert_eq!(rows[1].passing.len(), 1);
        assert_eq!(rows[1].lane, 1);
    }

    #[test]
    fn test_build_graph_octopus_merge_opens_a_lane_per_parent() {
        let rows = build_graph(&[
            commit("m", &["a", "b", "c"]),
            commit("a", &[]),
            commit("b", &[]),
            commit("c", &[]),
        ]);
        let targets: Vec<usize> = rows[0].outgoing.iter().map(|s| s.to).collect();
        assert_eq!(targets, vec![0, 1, 2]);
        assert_eq!(rows[0].width, 3);
    }

    /// a trunk of `merges` merge commits, each pulling in a one-commit side
    /// branch that forked from the next trunk commit
    fn merge_heavy_history(merges: usize) -> Vec<Commit> {
        let mut commits = Vec::with_capacity(merges * 2 + 1);
        for i in 0..merges {
            let next = format!("t{}", i + 1);
            let side = format!("s{i}");
            commits.push(Commit {
                hash: format!("t{i}"),
                ..commit("", &[&next, &side])
            });
            commits.push(Commit {
                hash: side,
                ..commit("", &[&next])
            });
        }
        commits.push(commit(&format!("t{merges}"), &[]));
        commits
    }

    #[test]
    fn test_build_graph_large_history_stays_two_lanes_wide() {
        let merges = 25_000;
        let rows = build_graph(&merge_heavy_history(merges));

        assert_eq!(rows.len(), merges * 2 + 1);
        assert!(rows.iter().all(|r| r.width <= 2));
        assert_eq!(rows.iter().filter(|r| r.is_merge).count(), merges);
        for (i, row) in rows.iter().enumerate() {
            let expected_lane = if i % 2 == 1 {
                1
            } else {
                0
            };
            assert_eq!(row.lane, expected_lane, "row {i}");
        }
        // every side branch closes back into the trunk
        assert_eq!(rows.last().map(|r| r.incoming.len()), Some(2));
    }

    /// lays `commits` out `page` at a time through one builder
    fn build_paged(commits: &[Commit], page: usize) -> Vec<GraphRow> {
        let mut builder = GraphBuilder::default();
        commits.chunks(page).flat_map(|chunk| builder.extend(chunk)).collect()
    }

    #[test]
    fn test_graph_builder_extend_paged_matches_single_pass() {
        let commits = merge_heavy_history(500);
        let whole = build_graph(&commits);
        for page in [1, 2, 3, 7, 200, 2000] {
            assert_eq!(build_paged(&commits, page), whole, "page size {page}");
        }
    }

    #[test]
    fn test_graph_builder_extend_lane_open_across_page_boundary() {
        // m's second parent s only arrives on the next page
        let commits = [
            commit("m", &["b", "s"]),
            commit("b", &["a"]),
            commit("s", &["a"]),
            commit("a", &[]),
        ];
        assert_eq!(build_paged(&commits, 2), build_graph(&commits));
    }

    #[test]
    fn test_graph_builder_extend_missing_parent_across_pages() {
        let commits = [
            commit("c", &["gone"]),
            commit("d", &[]),
            commit("e", &["d"]),
        ];
        assert_eq!(build_paged(&commits, 1), build_graph(&commits));
    }

    #[test]
    fn test_graph_builder_extend_empty_page_changes_nothing() {
        let commits = [commit("b", &["a"]), commit("a", &[])];
        let mut builder = GraphBuilder::default();
        let mut rows = builder.extend(&commits[..1]);
        assert!(builder.extend(&[]).is_empty());
        rows.extend(builder.extend(&commits[1..]));
        assert_eq!(rows, build_graph(&commits));
    }
}
