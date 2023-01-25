use clap::ArgMatches;

#[derive(Debug, Clone)]
pub enum Bounds {
    Rect { x1: i32, y1: i32, x2: i32, y2: i32 },
    Relative { dx: i32, dy: i32 },
}

impl Bounds {
    pub fn is_relative(&self) -> bool {
        match self {
            Bounds::Rect { .. } => false,
            Bounds::Relative { .. } => true,
        }
    }

    pub fn has_empty_range(&self) -> bool {
        match self {
            Bounds::Rect { x1, y1, x2, y2 } => x1 == x2 && y1 == y2,
            Bounds::Relative { dx, dy } => *dx == 0 && *dy == 0,
        }
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bounds::Rect { x1, y1, x2, y2 } => {
                write!(f, "absolute(p1=({x1}, {y1}), p2=({x2}, {y2}))")
            }
            Bounds::Relative { dx, dy } => write!(f, "relative(dx={dx}, dy={dy})"),
        }
    }
}

impl From<&ArgMatches> for Bounds {
    fn from(value: &ArgMatches) -> Self {
        if value.contains_id("absolute-bounds") {
            let coords = value
                .get_many::<i32>("absolute-bounds")
                .expect("values should be required by clap")
                .copied()
                .collect::<Vec<i32>>();

            return Bounds::Rect {
                x1: coords[0],
                y1: coords[1],
                x2: coords[2],
                y2: coords[3],
            };
        }
        if value.contains_id("relative-bounds") {
            let coords = value
                .get_many::<i32>("relative-bounds")
                .expect("values should be required by clap")
                .copied()
                .collect::<Vec<i32>>();

            return Bounds::Relative {
                dx: coords[0],
                dy: coords[1],
            };
        }

        Bounds::Relative { dx: 500, dy: 500 }
    }
}
