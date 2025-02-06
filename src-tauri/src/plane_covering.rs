use crate::common::{Point, Polygon};
use delaunator::triangulate;
use rand::Rng;

pub fn cover_rectangles(n: usize, w: f64, h: f64) -> Vec<Polygon> {
    let rows_cols = n.isqrt();
    let row_step: f64 = h / rows_cols as f64;
    let col_step: f64 = w / rows_cols as f64;
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

impl From<delaunator::Point> for Point {
    fn from(p: delaunator::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

pub fn cover_triangles(n: usize, w: f64, h: f64) -> Vec<Polygon> {
    let mut points = random_points_cell(n, w, h);
    points.append(&mut vec![
        delaunator::Point { x: 0.0, y: 0.0 },
        delaunator::Point { x: w, y: 0.0 },
        delaunator::Point { x: 0.0, y: h },
        delaunator::Point { x: w, y: h },
    ]);

    let triagulation = triangulate(&points);

    // Turn into polygons
    let triangles: Vec<_> = triagulation
        .triangles
        .chunks(3)
        .map(|t| Polygon {
            pnts: vec![
                points[t[0]].clone().into(),
                points[t[1]].clone().into(),
                points[t[2]].clone().into(),
            ],
        })
        .collect();

    log::debug!("Created {} triangles.", triangles.len());

    triangles
}

/// To improve the shape of the triangles, randomly distribute
/// points into cells instead of only on the plane itself.
/// Otherwise, we could end up with lengthy and pointy triangles.
fn random_points_cell(n: usize, w: f64, h: f64) -> Vec<delaunator::Point> {
    let mut rng = rand::thread_rng();

    let grid_cols = (n as f64).sqrt().ceil() as usize;
    let grid_rows = (n as f64 / grid_cols as f64).ceil() as usize;
    let cell_width = w / (grid_cols as f64);
    let cell_height = h / (grid_rows as f64);
    log::debug!("Using {grid_rows} rows and {grid_cols} cols.");

    let mut points = Vec::with_capacity(n);
    for row in 0..grid_rows {
        for col in 0..grid_cols {
            if points.len() >= n {
                break;
            }
            let base_x = col as f64 * cell_width;
            let base_y = row as f64 * cell_height;

            let offset_x = rng.gen_range(0.0..cell_width);
            let offset_y = rng.gen_range(0.0..cell_height);

            let x = base_x + offset_x;
            let y = base_y + offset_y;
            points.push(delaunator::Point { x, y });
        }
    }

    // If we haven't generated enough points, add random points
    while points.len() < n {
        points.push(delaunator::Point {
            x: rng.gen_range(0.0..w),
            y: rng.gen_range(0.0..h),
            });
    }

    points
}
