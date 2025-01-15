use crate::common::{Point, Polygon};

pub fn cover_rectangles(n: usize, w: f64, h: f64) -> Vec<Polygon> {
    let rows_cols = n.isqrt();
    let row_step: f64 = h.div_euclid(rows_cols as f64);
    let col_step: f64 = w.div_euclid(rows_cols as f64);
    itertools::iproduct!(0..rows_cols, 0..rows_cols)
        .map(|(row, col)| Polygon {
            pnts: vec![
                Point {
                    x: (col as f64) * col_step,
                    y: (row as f64) * row_step,
                },
                Point {
                    x: (col as f64) * col_step + col_step,
                    y: (row as f64) * row_step,
                },
                Point {
                    x: (col as f64) * col_step + col_step,
                    y: (row as f64) * row_step + row_step,
                },
                Point {
                    x: (col as f64) * col_step,
                    y: (row as f64) * row_step + row_step,
                },
            ],
        })
        .collect()
}

