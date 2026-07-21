//! Final-product output merger.
//!
//! After ghost routing places all trunk / tap / return belts, this
//! module merges the east-flowing output belts of rows producing the
//! same final product into a single south-facing splitter chain at
//! the bottom-right of the layout. Called once per final product at
//! the end of `route_bus_ghost` (Step 7).

use crate::models::{EntityDirection, PlacedEntity};
use crate::bus::balancer::splitter_for_belt;
use crate::bus::placer::RowSpan;

pub(crate) fn merge_output_rows(
    output_rows: &[usize],
    output_ys: &[i32],
    item: &str,
    row_spans: &[RowSpan],
    merge_start_y: i32,
    max_belt_tier: Option<&str>,
    min_merge_x: i32,
    blocked_columns: &[i32],
) -> (Vec<PlacedEntity>, i32, i32) {
    use crate::bus::balancer::underground_for_belt;
    use crate::common::{belt_entity_for_rate, ug_max_reach};

    debug_assert_eq!(
        output_rows.len(),
        output_ys.len(),
        "merge_output_rows: output_rows and output_ys must be 1:1 — output_ys[idx] is the \
         belt row for output_rows[idx] (RFC Fulgora D2a/D2b: primary rows use \
         `output_belt_y`, secondary-output rows use `secondary_output_belt`'s y)"
    );
    let mut entities: Vec<PlacedEntity> = Vec::new();
    let n = output_rows.len();
    if n == 0 {
        return (entities, merge_start_y, 0);
    }
    let merger_seg_id = Some(format!("merger:{}", item));

    // Calculate total rate
    let total_rate = output_rows.iter()
        .map(|&ri| {
            if ri >= row_spans.len() {
                0.0
            } else {
                row_spans[ri].spec.outputs.iter()
                    .filter(|o| o.item == item)
                    .map(|o| o.rate * row_spans[ri].machine_count as f64)
                    .sum::<f64>()
            }
        })
        .sum::<f64>();

    let belt_name = belt_entity_for_rate(total_rate * 2.0, max_belt_tier);
    let splitter_name = splitter_for_belt(belt_name);

    // Column position: east of the widest participating row, but never west
    // of `min_merge_x` — the caller threads a running cursor across
    // successive per-item merges so two output items' splitter cascades and
    // south columns tile left-to-right instead of stamping the same tiles
    // (multi-item solid output support, Phase 2 of rfc-solver-net-flow).
    let merge_x = (output_rows.iter()
        .map(|&ri| if ri < row_spans.len() { row_spans[ri].row_width } else { 0 })
        .max()
        .unwrap_or(0) + 1)
        .max(min_merge_x);

    for (idx, &ri) in output_rows.iter().enumerate() {
        if ri >= row_spans.len() {
            continue;
        }
        let out_y = output_ys[idx];
        let col_x = merge_x + (n - 1 - idx) as i32; // first row rightmost, last row at merge_x

        // Extend EAST belts from the row's rightmost tile to the merge
        // column. Earlier items' south columns (`blocked_columns`) lie in
        // this run's path — bridge each contiguous blocked range with an
        // underground pair instead of stamping over (or sideloading into)
        // the foreign belt.
        let rw = row_spans[ri].row_width;
        let ug_name = underground_for_belt(belt_name);
        let reach = ug_max_reach(belt_name) as i32;
        let mut x = rw;
        while x < col_x {
            if blocked_columns.contains(&x) {
                // Contiguous blocked run [x, run_end], clamped by UG reach
                // (entrance at x-1, exit at run_end+1; gap ≤ reach).
                let mut run_end = x;
                while run_end + 1 < col_x
                    && blocked_columns.contains(&(run_end + 1))
                    && (run_end + 1) - x < reach
                {
                    run_end += 1;
                }
                debug_assert!(x > rw, "no room for UG entrance before blocked column");
                // Replace the belt stamped at x-1 with a UG entrance.
                if let Some(prev) = entities
                    .iter_mut()
                    .rev()
                    .find(|e| e.x == x - 1 && e.y == out_y)
                {
                    prev.name = ug_name.to_string();
                    prev.io_type = Some("input".to_string());
                }
                entities.push(PlacedEntity {
                    name: ug_name.to_string(),
                    x: run_end + 1,
                    y: out_y,
                    direction: EntityDirection::East,
                    io_type: Some("output".to_string()),
                    carries: Some(item.to_string()),
                    segment_id: merger_seg_id.clone(),
                    rate: Some(total_rate),
                    ..Default::default()
                });
                x = run_end + 2;
                continue;
            }
            entities.push(PlacedEntity {
                name: belt_name.to_string(),
                x,
                y: out_y,
                direction: EntityDirection::East,
                carries: Some(item.to_string()),
                segment_id: merger_seg_id.clone(),
                rate: Some(total_rate),
                ..Default::default()
            });
            x += 1;
        }

        // SOUTH column from out_y to merge_start_y.
        for y in out_y..merge_start_y {
            entities.push(PlacedEntity {
                name: belt_name.to_string(),
                x: col_x,
                y,
                direction: EntityDirection::South,
                carries: Some(item.to_string()),
                segment_id: merger_seg_id.clone(),
                rate: Some(total_rate),
                ..Default::default()
            });
        }
    }

    // Sequential splitter cascade merging N south columns into 1.
    // Columns are at x = merge_x (row n-1) through merge_x + n-1 (row 0).
    //
    // At each step we place a SOUTH splitter that merges two adjacent columns.
    // A SOUTH splitter at (x, y) spans tiles (x, y) and (x+1, y), accepting
    // input from (x, y-1) and (x+1, y-1), outputting at (x, y+1) and (x+1, y+1).
    // We use the left output (x) and discard the right.
    //
    // Between steps, ALL surviving columns need a continuation belt at each row
    // so they stay connected through to the next splitter.
    let mut y_cursor = merge_start_y;
    // Active columns, sorted left-to-right.
    let mut active: Vec<i32> = (0..n as i32).map(|i| merge_x + i).collect();

    while active.len() > 1 {
        let right_x = active.pop().unwrap();
        let left_x = *active.last().unwrap();

        // Splitter merging left_x and left_x+1 (right_x should equal left_x+1)
        // If not adjacent, route right column west first.
        if right_x != left_x + 1 {
            for x in ((left_x + 2)..=right_x).rev() {
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x,
                    y: y_cursor,
                    direction: EntityDirection::West,
                    carries: Some(item.to_string()),
                    segment_id: merger_seg_id.clone(),
                    rate: Some(total_rate),
                    ..Default::default()
                });
            }
        }
        // Pass-through belts at the splitter row for uninvolved columns.
        // The splitter occupies (left_x, y_cursor) and (left_x+1, y_cursor).
        for &ax in &active {
            if ax != left_x && ax != left_x + 1 {
                entities.push(PlacedEntity {
                    name: belt_name.to_string(),
                    x: ax,
                    y: y_cursor,
                    direction: EntityDirection::South,
                    carries: Some(item.to_string()),
                    segment_id: merger_seg_id.clone(),
                    rate: Some(total_rate),
                    ..Default::default()
                });
            }
        }
        entities.push(PlacedEntity {
            name: splitter_name.to_string(),
            x: left_x,
            y: y_cursor,
            direction: EntityDirection::South,
            carries: Some(item.to_string()),
            segment_id: merger_seg_id.clone(),
            rate: Some(total_rate),
            ..Default::default()
        });
        y_cursor += 1;

        // Continuation belts below the splitter for all surviving columns.
        for &ax in &active {
            entities.push(PlacedEntity {
                name: belt_name.to_string(),
                x: ax,
                y: y_cursor,
                direction: EntityDirection::South,
                carries: Some(item.to_string()),
                segment_id: merger_seg_id.clone(),
                rate: Some(total_rate),
                ..Default::default()
            });
        }
        y_cursor += 1;
    }

    (entities, y_cursor, merge_x + n as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ItemFlow, MachineSpec};

    fn make_test_row_span(
        recipe: &str,
        y_start: i32,
        inputs: Vec<ItemFlow>,
        outputs: Vec<ItemFlow>,
        machine_count: usize,
        input_belt_y: Vec<i32>,
    ) -> RowSpan {
        RowSpan {
            y_start,
            y_end: y_start + 3,
            spec: MachineSpec {
                entity: "assembling-machine-3".to_string(),
                recipe: recipe.to_string(),
                self_loop: vec![], voider: false, game_modules: Vec::new(),
                count: machine_count as f64,
                inputs,
                outputs,
            },
            machine_count,
            module_id: 0,
            input_belt_y,
            output_belt_y: y_start + 2,
            row_width: 10,
            fluid_port_ys: Vec::new(),
            fluid_port_pipes: Vec::new(),
            fluid_output_port_pipes: Vec::new(),
            output_east: true,
            output_belt_x_min: 0,
            output_belt_x_max: 9,
            horizontal_stack: None,
            secondary_output_belt: None,
            sorted_output_belts: Vec::new(),
        }
    }


    /// Phase 2 (rfc-solver-net-flow): two output items' merge blocks must
    /// tile east via the threaded cursor instead of stamping the same
    /// tiles. Regression for the review finding that per-item merge_x was
    /// computed independently.
    #[test]
    fn test_two_items_merge_blocks_do_not_overlap() {
        use rustc_hash::FxHashSet;
        let row0 = make_test_row_span(
            "iron-gear-wheel",
            0,
            vec![],
            vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            2,
            vec![0],
        );
        let row1 = make_test_row_span(
            "iron-stick",
            5,
            vec![],
            vec![ItemFlow { item: "iron-stick".to_string(), rate: 2.0, is_fluid: false, module_id: 0 }],
            2,
            vec![5],
        );
        let rows = [row0, row1];
        let (a_ents, a_end_y, a_max_x) = merge_output_rows(&[0], &[rows[0].output_belt_y], "iron-gear-wheel", &rows, 15, None, 11, &[]);
        // Caller threads: next min_merge_x = returned max_x + 1, start_y = max_y.
        let blocked: Vec<i32> = ((a_max_x - 1)..a_max_x).collect();
        let (b_ents, _b_end_y, b_max_x) = merge_output_rows(&[1], &[rows[1].output_belt_y], "iron-stick", &rows, a_end_y.max(15), None, a_max_x + 1, &blocked);
        assert!(b_max_x > a_max_x);
        let a_tiles: FxHashSet<(i32, i32)> = a_ents.iter().map(|e| (e.x, e.y)).collect();
        let overlap: Vec<(i32, i32)> = b_ents
            .iter()
            .map(|e| (e.x, e.y))
            .filter(|t| a_tiles.contains(t))
            .collect();
        assert!(overlap.is_empty(), "merge blocks overlap at {overlap:?}");
        // And without the cursor they WOULD overlap (guard the guard):
        let (c_ents, _c_end_y, _c) = merge_output_rows(&[1], &[rows[1].output_belt_y], "iron-stick", &rows, 15, None, 0, &[]);
        let c_overlap = c_ents.iter().map(|e| (e.x, e.y)).any(|t| a_tiles.contains(&t));
        assert!(c_overlap, "expected uncursored merges to collide — geometry changed?");
    }

    #[test]
    fn test_merge_output_rows_single_row() {
        let row_span = make_test_row_span(
            "iron-plate",
            0,
            vec![],
            vec![ItemFlow { item: "iron-plate".to_string(), rate: 10.0, is_fluid: false, module_id: 0 }],
            1,
            vec![],
        );

        let output_rows = vec![0];
        let output_ys = vec![row_span.output_belt_y];
        let (entities, _end_y, _merge_max_x) = merge_output_rows(&output_rows, &output_ys, "iron-plate", &[row_span], 20, None, 0, &[]);

        // Single row should extend EAST and SOUTH without splitters
        assert!(!entities.is_empty());
        assert!(entities.iter().all(|e| e.carries.as_deref() == Some("iron-plate")));
    }

    #[test]
    fn test_merge_output_rows_multiple_rows() {
        let row_span1 = make_test_row_span(
            "iron-plate",
            0,
            vec![],
            vec![ItemFlow { item: "iron-plate".to_string(), rate: 10.0, is_fluid: false, module_id: 0 }],
            1,
            vec![],
        );
        let row_span2 = make_test_row_span(
            "iron-plate",
            0,
            vec![],
            vec![ItemFlow { item: "iron-plate".to_string(), rate: 10.0, is_fluid: false, module_id: 0 }],
            1,
            vec![],
        );

        let output_rows = vec![0, 1];
        let output_ys = vec![row_span1.output_belt_y, row_span2.output_belt_y];
        let (entities, _end_y, _merge_max_x) = merge_output_rows(&output_rows, &output_ys, "iron-plate", &[row_span1, row_span2], 20, None, 0, &[]);

        // Multiple rows should include splitters
        let splitters = entities.iter().filter(|e| e.name.contains("splitter")).count();
        assert!(splitters > 0, "Expected splitters for multiple rows");
    }

    #[test]
    fn test_merge_output_rows_two_rows_have_splitters_and_correct_item() {
        // Two rows producing iron-gear-wheel: the merger must emit splitters and
        // all entities must carry iron-gear-wheel.
        let row0 = {
            let mut rs = make_test_row_span(
                "iron-gear-wheel",
                0,
                vec![],
                vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 5.0, is_fluid: false, module_id: 0 }],
                2,
                vec![],
            );
            rs.output_belt_y = 2;
            rs.row_width = 8;
            rs
        };
        let row1 = {
            let mut rs = make_test_row_span(
                "iron-gear-wheel",
                5,
                vec![],
                vec![ItemFlow { item: "iron-gear-wheel".to_string(), rate: 5.0, is_fluid: false, module_id: 0 }],
                2,
                vec![],
            );
            rs.output_belt_y = 7;
            rs.row_width = 8;
            rs
        };

        let output_ys = vec![row0.output_belt_y, row1.output_belt_y];
        let (entities, end_y, merge_max_x) = merge_output_rows(
            &[0, 1],
            &output_ys,
            "iron-gear-wheel",
            &[row0, row1],
            15,
            None,
            0,
            &[],
        );

        // Splitters must be present
        let splitters: Vec<_> = entities.iter()
            .filter(|e| e.name.contains("splitter"))
            .collect();
        assert!(!splitters.is_empty(), "Expected splitter(s) in merger for 2 rows");

        // Every entity must carry the correct item
        for e in &entities {
            assert_eq!(
                e.carries.as_deref(),
                Some("iron-gear-wheel"),
                "All merger entities should carry iron-gear-wheel, got {:?}",
                e
            );
        }

        // end_y and merge_max_x should be sane
        assert!(end_y > 15, "end_y should be greater than merge_start_y");
        assert!(merge_max_x > 0, "merge_max_x should be positive");
    }

    #[test]
    fn test_merge_output_rows_splitters_face_south() {
        // Splitters produced by merge_output_rows should face SOUTH (merging
        // parallel SOUTH-flowing trunks).
        let row0 = make_test_row_span(
            "electronic-circuit",
            0,
            vec![],
            vec![ItemFlow { item: "electronic-circuit".to_string(), rate: 5.0, is_fluid: false, module_id: 0 }],
            1,
            vec![],
        );
        let row1 = make_test_row_span(
            "electronic-circuit",
            5,
            vec![],
            vec![ItemFlow { item: "electronic-circuit".to_string(), rate: 5.0, is_fluid: false, module_id: 0 }],
            1,
            vec![],
        );

        let output_ys = vec![row0.output_belt_y, row1.output_belt_y];
        let (entities, _end_y, _merge_max_x) = merge_output_rows(
            &[0, 1],
            &output_ys,
            "electronic-circuit",
            &[row0, row1],
            20,
            None,
            0,
            &[],
        );

        let splitters: Vec<_> = entities.iter().filter(|e| e.name.contains("splitter")).collect();
        for s in &splitters {
            assert_eq!(s.direction, EntityDirection::South, "Merger splitters should face SOUTH");
        }
    }

    // -----------------------------------------------------------------------
    // plan_bus_lanes via solver - integration
    // -----------------------------------------------------------------------
}
