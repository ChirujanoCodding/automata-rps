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
