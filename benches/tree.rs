use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use lustre::{hittables::Hittable, scenes, tree::Tree};
use rand::SeedableRng;

fn bench_gen(c: &mut Criterion) {
    // configuration of criterion
    let mut bench_group = c.benchmark_group("tree_gen");
    // filter noise more noise
    bench_group.noise_threshold(0.05);
    // smaller sig level to combat noise
    bench_group.significance_level(0.1);
    // more samples -> more precision
    bench_group.sample_size(1000);

    // modify scene selection here
    let scenes_to_check = [scenes::SceneType::CoverPhoto, scenes::SceneType::CornellBox];

    // check against each chosen scene
    for scene in scenes_to_check {
        // configuration of Tree input
        let scene_name = format!("{scene:?}");
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let geo = scenes::get_geometry(scene, &mut rng, 0.0..1.0);

        // use bench with input for cleaner per-scene test name
        bench_group.bench_with_input(BenchmarkId::from_parameter(scene_name), &geo, |b, s| {
            // no need for iter_batched since we don't modify the input
            b.iter(|| Tree::new(s.clone(), 0.0, 1.0))
        });
    }

    bench_group.finish();
}

fn bench_hit(c: &mut Criterion) {
    // configuration of criterion
    let mut bench_group = c.benchmark_group("tree_intersect");
    // more samples -> more precision
    bench_group.sample_size(1000);

    // modify scene selection here
    let scenes_to_check = [scenes::SceneType::CoverPhoto, scenes::SceneType::DebugFinal];

    // check against each chosen scene
    for scene in scenes_to_check {
        // tree configuration

        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let geo = scenes::get_geometry(scene, &mut rng, 0.0..1.0);
        let tree = Tree::new(geo, 0.0, 1.0);

        // ray configuration
        // viewport@ (0.5,0.5) is center of the image. should be guaranteed to hit with specific scenes
        let ray = lustre::scenes::get_camera(scene).get_ray(0.5, 0.5, &mut rng);

        // correctness check
        assert!(
            tree.hit(&ray, 0.001, f32::INFINITY).is_some(),
            "ray directed at the center of the scene should hit the scene"
        );

        // benchmark
        let scene_name = format!("{scene:?}");
        bench_group.bench_function(BenchmarkId::new("definite_hit", scene_name), |b| {
            b.iter(|| tree.hit(&ray, 0.001, f32::INFINITY))
        });
    }
}

fn bench_miss(c: &mut Criterion) {
    // configuration of criterion
    let mut bench_group = c.benchmark_group("tree_intersect");
    // more samples -> more precision
    bench_group.sample_size(1000);

    // modify scene selection here
    let scenes_to_check = [scenes::SceneType::CoverPhoto, scenes::SceneType::DebugFinal];

    // check against each chosen scene
    for scene in scenes_to_check {
        // tree configuration
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let geo = scenes::get_geometry(scene, &mut rng, 0.0..1.0);
        let tree = Tree::new(geo, 0.0, 1.0);

        // ray configuration
        let mut ray = lustre::scenes::get_camera(scene).get_ray(0.5, 0.5, &mut rng);
        // negating all components "flips" the vector
        ray.direction *= -1.0;

        // correctness check
        assert!(
            tree.hit(&ray, 0.001, f32::INFINITY).is_none(),
            "ray directed away from the scene should not hit the scene"
        );

        // benchmark
        let scene_name = format!("{scene:?}");
        bench_group.bench_function(BenchmarkId::new("definite_miss", scene_name), |b| {
            b.iter(|| tree.hit(&ray, 0.001, f32::INFINITY))
        });
    }
}

criterion_group! {benches, bench_gen, bench_hit, bench_miss}
criterion_main!(benches);