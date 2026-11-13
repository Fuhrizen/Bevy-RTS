use bevy::prelude::*;

fn main() {
    App::new().add_plugins(DefaultPlugins).run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bevy_app_initializes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.update();
    }
}
