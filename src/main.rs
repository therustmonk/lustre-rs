use rand::SeedableRng;

use lustre::{
    cli::{Arguments, Parser},
    render, scenes,
    tree::Tree,
};

fn main() {
    // Parsing cli args
    let cli_args = Arguments::parse();

    // set up enviroment
    let mut rng = if let Some(seed) = cli_args.seed {
        // use user-provided seed if available
        rand::rngs::SmallRng::seed_from_u64(seed)
    } else if cfg!(debug_assertions) {
        // if debugging, use deterministic seed
        rand::rngs::SmallRng::seed_from_u64(0)
    } else {
        // otherwise real psuedo-randomness
        rand::rngs::SmallRng::from_entropy()
    };

    // Get scene
    let (world, cam, dimensions) =
        scenes::get_scene(cli_args.image_width, cli_args.scene, &mut rng);
    let world = Tree::new(world, cam.shutter_time.start, cam.shutter_time.end);

    let renderer = render::Renderer::new(
        dimensions.0,
        dimensions.1,
        cli_args.samples_per_pixel,
        cli_args.bounce_depth,
    );

    let img_buf = renderer.render_scene(cam, world);

    // write image to file
    match img_buf.save(&cli_args.output) {
        Ok(()) => println!("Image written to {:?}", &cli_args.output),
        Err(why) => {
            eprintln!("Failed to write: {}", why);
        }
    }
}
