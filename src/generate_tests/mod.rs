use crate::engine::scripts::Instruction;
use crate::engine::scripts::{
    Function, FunctionName, IsolineAction, Script, SnapshotAction,
};
use crate::erode::Parameters;
use crate::heightmap::{HeightmapParameters, HeightmapType, ProceduralHeightmapSettings};
use crate::partitioning::{Method, GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS, GAUSSIAN_DEFAULT_SIGMA};
use crate::visualize::events::UiEvent;
use crate::visualize::wrappers::{FractalTypeWrapper, NoiseTypeWrapper};
use std::default::Default;

pub struct Test {
    script: Script,
    uid: u64,
}

fn methods(grid_sizes: &Vec<usize>) -> Vec<Method> {
    let mut methods = vec![];
    for size in grid_sizes {
        methods.push(Method::Subdivision(*size));
        methods.push(Method::SubdivisionBlurBoundary((
            *size,
            (GAUSSIAN_DEFAULT_SIGMA, GAUSSIAN_DEFAULT_BOUNDARY_THICKNESS),
        )));
        methods.push(Method::GridOverlapBlend(*size));
    }
    methods.push(Method::Default);
    methods
}

fn generate_heightmap_types(resolutions: &Vec<usize>) -> Vec<HeightmapType> {
    let mut types = Vec::new();
    for seed in 1000..1010 {
        // let seed;
        let noise_type = NoiseTypeWrapper::PerlinFractal;
        let fractal_type = FractalTypeWrapper::FBM;
        // let fractal_octaves;
        // let fractal_gain;
        // let fractal_lacunarity;
        // let frequency;
        for fractal_octaves in 7..8 {
            for fractal_gain in (3..5i8).map(|n| f32::from(n) / 10.0) {
                for fractal_lacunarity in (2..3i8).map(|n| f32::from(n)) {
                    for frequency in (2..30i8).step_by(5).map(|n| f32::from(n) / 10.0).rev() {
                        for res in resolutions.iter() {
                            let params = HeightmapParameters { size: *res };
                            types.push(HeightmapType::Procedural(
                                params,
                                ProceduralHeightmapSettings {
                                    seed,
                                    noise_type,
                                    fractal_type,
                                    fractal_octaves,
                                    fractal_gain,
                                    fractal_lacunarity,
                                    frequency,
                                },
                            ))
                        }
                    }
                }
            }
        }
    }
    types
}

pub fn find_last_iteration() -> usize {
    let mut paths = std::fs::read_dir("./")
        .unwrap()
        .filter_map(|path| {
            let path = path.unwrap();
            if path.path().is_dir() {
                None
            } else {
                let p = path.path().display().to_string();
                Some(p.split("-").nth(2)?.strip_prefix("i")?.parse().ok()?)
            }
        })
        .collect::<Vec<usize>>();
    paths.sort();
    *paths.last().unwrap()
}

pub fn generate_all_permutations() -> Script {
    let skip = find_last_iteration();
    let mut test = Test::new(HeightmapType::default());

    let grid_sizes = vec![128, 64, 32, 16, 8, 4];
    let methods = methods(&grid_sizes);
    // let resolutions: Vec<usize> = vec![128, 256, 512, 1024].into_iter().rev().collect();
    let resolutions: Vec<usize> = vec![128, 256, 512, 1024];
    let map_types = generate_heightmap_types(&resolutions);

    let total_iterations = map_types.len() * methods.len() * 100;

    for (i, map) in map_types.into_iter().enumerate().skip(skip) {
        test = test
            .run(Instruction::NewState(map))
            .run(Instruction::SetAdvancedView(false));
        for (j, method) in methods.iter().enumerate() {
            let iterations = (i * methods.len() + j) * 100;
            test = test
                .run(Instruction::Queue(UiEvent::ReplaceHeightmap))
                // .run(Instruction::Queue(UiEvent::ExportActiveHeightmap))
                .run(Instruction::Queue(UiEvent::SelectMethod(*method)))
                .run(Instruction::Flush)
                .run(Instruction::SetErosionParameters(Parameters {
                    num_iterations: 200 * map.params().size,
                    ..Default::default()
                }))
                .run(Instruction::Queue(UiEvent::RunSimulation))
                // .run(Instruction::Handover) // works with this line wtf
                .run(Instruction::Render(true)) // works with this line wtf
                // .run(Instruction::Render(false)) // but not with this
                .run(Instruction::Flush)
                // .run(Instruction::Queue(UiEvent::ExportActiveHeightmap))
                .run(Instruction::Print(format!(
                    "{} / {}  --> {}%",
                    iterations,
                    total_iterations,
                    100.0 * iterations as f32 / total_iterations as f32
                )));
            println!(
                "{} / {}  --> {}% {}",
                iterations,
                total_iterations,
                100.0 * iterations as f32 / total_iterations as f32,
                map.params().size
            );

            for value in (0..10).map(|n: i8| f32::from(n) / 10.0) {
                test = test.run(Instruction::Isoline(IsolineAction::SetValue(value)));
                for error in (0..10).map(|n: i8| f32::from(n) / 10.0) {
                    test = test
                        .run(Instruction::Isoline(IsolineAction::SetError(error)))
                        .run(Instruction::Isoline(IsolineAction::Queue))
                        // .run(Instruction::Queue(UiEvent::ExportActiveHeightmap))
                        .run(Instruction::SetName(format!(
                            "iteration-{iterations}-i-{i}-value-{value}-error-{error}"
                        )))
                        .run(Instruction::Flush)
                        .run(Instruction::Snapshot(SnapshotAction::Take))
                        // .run(Instruction::Handover)
                        .run(Instruction::Render(true));
                }
            }

            test = test
                .run(Instruction::Print(format!(
                    "saving: iteration-{iterations}-i{i}-j{j} method-{}",
                    method.to_string()
                )))
                .save(&format!(
                    "iteration-{iterations}-i{i}-j{j} method-{}",
                    method.to_string()
                ));
        }
    }

    test.run(Instruction::Handover).script
}

pub fn generate_test() -> Script {
    let min_size = 256;
    let max_size = 1024;
    let step_size = (max_size - min_size) / 4;
    Test::new(HeightmapType::default())
        .generate_procedural_tests(4, min_size, max_size, step_size)
        .inject(
            "finish".to_string(),
            vec![Instruction::Print("All Done!".to_string())],
        )
        .script
}

impl Test {
    fn new(initial_state: HeightmapType) -> Self {
        let main_name = "main".to_string();
        let main = vec![Instruction::NewState(initial_state)];
        let mut script = Script::new();

        script.insert(main_name, main);
        Test { script, uid: 0 }
    }

    fn push(mut self) -> Self {
        self.script
            .get_mut("main")
            .unwrap()
            .push(Instruction::PushState);
        self
    }

    fn pop(mut self) -> Self {
        self.script
            .get_mut("main")
            .unwrap()
            .push(Instruction::PopSetState);
        self
    }

    fn function_generate_resolution(size: usize) -> Function {
        vec![
            Instruction::Size(size),
            Instruction::Queue(UiEvent::ReplaceHeightmap),
            Instruction::Isoline(IsolineAction::Queue),
            Instruction::Flush,
        ]
    }

    fn generate_resolutions(
        self,
        min_size: usize,
        max_size: usize,
        step_by: usize,
        intermediate: fn(usize) -> Vec<Function>,
    ) -> Self {
        let mut function = Vec::new();

        for size in (min_size..=max_size).step_by(step_by) {
            function.append(&mut Self::function_generate_resolution(size));
            for mut f in intermediate(size) {
                function.append(&mut f)
            }
        }

        self.inject("generate-resolutions".to_string(), function)
    }

    fn generate_resolution_erosion_tests(
        self,
        min_size: usize,
        max_size: usize,
        step_by: usize,
        uid: u64,
    ) -> Self {
        self.push()
            .name(&format!("grid_overlap_{}", uid))
            .generate_resolutions(min_size, max_size, step_by, |_size| {
                vec![
                    Test::function_erode(Method::GridOverlapBlend(6)),
                    Test::function_collect_data(),
                ]
            })
            .save(&format!("grid_overlap_{}", uid))
            .pop()
            .push()
            .name(&format!("subdivision_{}", uid))
            .generate_resolutions(min_size, max_size, step_by, |_| {
                vec![
                    Test::function_erode(Method::Subdivision(6)),
                    Test::function_collect_data(),
                ]
            })
            .save(&format!("subdivision_{}", uid))
            .pop()
            .push()
            .name(&format!("standard_{}", uid))
            .generate_resolutions(min_size, max_size, step_by, |_| {
                vec![
                    Test::function_erode(Method::Default),
                    Test::function_collect_data(),
                ]
            })
            .save(&format!("standard_{}", uid))
            .pop()
    }

    fn generate_procedural_test(
        mut self,
        min_size: usize,
        max_size: usize,
        step_size: usize,
    ) -> Self {
        let uid = self.get_uid();
        self.heightmap(HeightmapType::Procedural(
            HeightmapParameters::default(),
            ProceduralHeightmapSettings {
                seed: 1337 + uid,
                noise_type: NoiseTypeWrapper::PerlinFractal,
                ..Default::default()
            },
        ))
        .generate_resolution_erosion_tests(min_size, max_size, step_size, uid)
    }

    fn generate_procedural_tests(
        mut self,
        n: usize,
        min_size: usize,
        max_size: usize,
        step_size: usize,
    ) -> Self {
        for _ in 0..n {
            self = self.generate_procedural_test(min_size, max_size, step_size);
        }
        self
    }

    fn function_erode(method: Method) -> Function {
        vec![
            Instruction::Queue(UiEvent::SelectMethod(method)),
            Instruction::Queue(UiEvent::RunSimulation),
            Instruction::Flush,
        ]
    }

    fn function_collect_data() -> Function {
        vec![
            Instruction::Queue(UiEvent::ExportActiveHeightmap),
            Instruction::Isoline(IsolineAction::Queue),
            Instruction::Queue(UiEvent::ExportActiveHeightmap),
            Instruction::Flush,
            Instruction::Snapshot(SnapshotAction::Take),
            // Instruction::Render(false),
        ]
    }

    fn heightmap(self, heightmap: HeightmapType) -> Self {
        self.run(Instruction::NewState(heightmap))
    }

    fn name(self, name: &str) -> Self {
        self.run(Instruction::SetName(name.to_string()))
    }

    fn save(self, filename: &str) -> Self {
        self.run(Instruction::Snapshot(SnapshotAction::SaveAndClear(
            format!("{}.json", filename),
        )))
    }

    fn run(mut self, instruction: Instruction) -> Self {
        self.script.get_mut("main").unwrap().push(instruction);
        self
    }

    fn inject(mut self, name: FunctionName, function: Function) -> Self {
        let injection_name = self.get_injection_name(name);
        self.script
            .get_mut("main")
            .unwrap()
            .push(Instruction::Call(injection_name.clone()));
        self.script.insert(injection_name, function);
        self
    }

    fn get_uid(&mut self) -> u64 {
        let uid = self.uid;
        self.uid += 1;
        uid
    }

    fn get_injection_name(&self, function_name: FunctionName) -> FunctionName {
        format!("{}-{}", function_name, self.script.len())
    }
}
