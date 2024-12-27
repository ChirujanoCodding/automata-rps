use rand::Rng;

pub fn on_generic_borders(entity: f32, borders: f32) -> bool {
    entity.abs() >= borders.abs()
}

pub fn on_borders(entity: (f32, f32), borders: (f32, f32)) -> bool {
    let (x, y) = entity;
    let (width, height) = borders;
    on_generic_borders(x, width) || on_generic_borders(y, height)
}

#[macro_export]
macro_rules! add_components {
    ($entity:ident, $enemy:ident, $target:ident) => {
        impl HasEnemy for $entity {
            type Enemy = $enemy;
        }

        impl HasTarget for $entity {
            type Target = $target;
        }
    };
}

pub fn generate_regions(width: f32, height: f32, count: usize) -> Vec<(f32, f32, f32)> {
    let mut rng = rand::thread_rng();
    let mut regions = Vec::new();

    let radius = 60.;

    let width = width - radius;
    let height = height - radius;

    while regions.len() < count {
        let x = rng.gen_range(-width..=width);
        let y = rng.gen_range(-height..=height);
        regions.push((x, y, radius));
    }

    regions
}
