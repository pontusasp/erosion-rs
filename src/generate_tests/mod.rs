use std::default::Default;
use crate::engine::scripts::{Function, FunctionName, IsolineAction, Script, SnapshotAction};
use crate::heightmap::{HeightmapParameters, HeightmapType, ProceduralHeightmapSettings};
use crate::engine::scripts::Instruction;
use crate::partitioning::Method;
use crate::visualize::events::UiEvent;
use crate::visualize::wrappers::NoiseTypeWrapper;

pub struct Test {
    script: Script,
    uid: u64,
}

pub fn generate() -> Script {
    let min_size = 256;
    let max_size = 1024;
    let step_size = (max_size - min_size) / 4;
    Test::new(HeightmapType::default())
        .generate_procedural_tests(4, min_size, max_size, step_size)
        .inject("finish".to_string(), vec![
            Instruction::Print("All Done!".to_string())
        ])
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
        self.script.get_mut("main").unwrap().push(Instruction::PushState);
        self
    }

    fn pop(mut self) -> Self {
        self.script.get_mut("main").unwrap().push(Instruction::PopSetState);
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

    fn generate_resolutions(self, min_size: usize, max_size: usize, step_by: usize, intermediate: fn(usize) -> Vec<Function>) -> Self {
        let mut function = Vec::new();

        for size in (min_size..=max_size).step_by(step_by) {
            function.append(&mut Self::function_generate_resolution(size));
            for mut f in intermediate(size) {
                function.append(&mut f)
            }
        }

        self.inject("generate-resolutions".to_string(), function)
    }

    fn generate_resolution_erosion_tests(self, min_size: usize, max_size: usize, step_by: usize, uid: u64) -> Self {
        self.push()
            .name(&format!("grid_overlap_{}", uid))
            .generate_resolutions(min_size, max_size, step_by, |size| {
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

    fn generate_procedural_test(mut self, min_size: usize, max_size: usize, step_size: usize) -> Self {
        let uid = self.get_uid();
        self.heightmap(HeightmapType::Procedural(
            HeightmapParameters::default(),
            ProceduralHeightmapSettings {
                seed: 1337 + uid,
                noise_type: NoiseTypeWrapper::PerlinFractal,
                ..Default::default()
            }
        )).generate_resolution_erosion_tests(min_size, max_size, step_size, uid)
    }

    fn generate_procedural_tests(mut self, n: usize, min_size: usize, max_size: usize, step_size: usize) -> Self {
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
            Instruction::Render(false),
        ]
    }

    fn heightmap(self, heightmap: HeightmapType) -> Self {
        self.run(Instruction::NewState(heightmap))
    }

    fn name(self, name: &str) -> Self {
        self.run(Instruction::SetName(name.to_string()))
    }

    fn save(self, filename: &str) -> Self {
        self.run(Instruction::Snapshot(SnapshotAction::SaveAndClear(format!("{}.json", filename))))
    }

    fn run(mut self, instruction: Instruction) -> Self {
        self.script.get_mut("main").unwrap().push(instruction);
        self
    }

    fn inject(mut self, name: FunctionName, function: Function) -> Self {
        let injection_name = self.get_injection_name(name);
        self.script.get_mut("main").unwrap().push(Instruction::Call(injection_name.clone()));
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

