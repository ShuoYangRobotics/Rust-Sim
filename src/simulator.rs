use rapier2d::prelude::*;
use std::time::Instant;

pub struct Simulator {
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    ball1_handle: RigidBodyHandle,
    ball2_handle: RigidBodyHandle,
}

impl Simulator {
    pub fn new(
        ball1_config: (f32, f32, f32, [f32; 2], [f32; 2]),
        ball2_config: (f32, f32, f32, [f32; 2], [f32; 2]),
    ) -> Self {
        let gravity = vector![0.0, -9.81];
        let integration_parameters = IntegrationParameters::default();
        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = DefaultBroadPhase::new(); // using concrete type
        let mut narrow_phase = NarrowPhase::new();
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut multibody_joint_set = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
        let mut query_pipeline = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();

        // Create the ground
        let ground_size = 10.0;
        let ground_height = 0.1;
        let ground = RigidBodyBuilder::fixed()
            .translation(vector![0.0, -ground_height])
            .build();
        let ground_handle = rigid_body_set.insert(ground);
        let ground_collider = ColliderBuilder::cuboid(ground_size, ground_height).build();
        collider_set.insert_with_parent(ground_collider, ground_handle, &mut rigid_body_set);

        // Create the balls
        let (ball1_radius, ball1_mass, ball1_elasticity, ball1_position, ball1_velocity) =
            ball1_config;
        let ball1 = RigidBodyBuilder::dynamic()
            .translation(vector![ball1_position[0], ball1_position[1]])
            .linvel(vector![ball1_velocity[0], ball1_velocity[1]])
            .build();
        let ball1_handle = rigid_body_set.insert(ball1);
        let ball1_collider = ColliderBuilder::ball(ball1_radius)
            .restitution(ball1_elasticity)
            .density(ball1_mass)
            .build();
        collider_set.insert_with_parent(ball1_collider, ball1_handle, &mut rigid_body_set);

        let (ball2_radius, ball2_mass, ball2_elasticity, ball2_position, ball2_velocity) =
            ball2_config;
        let ball2 = RigidBodyBuilder::dynamic()
            .translation(vector![ball2_position[0], ball2_position[1]])
            .linvel(vector![ball2_velocity[0], ball2_velocity[1]])
            .build();
        let ball2_handle = rigid_body_set.insert(ball2);
        let ball2_collider = ColliderBuilder::ball(ball2_radius)
            .restitution(ball2_elasticity)
            .density(ball2_mass)
            .build();
        collider_set.insert_with_parent(ball2_collider, ball2_handle, &mut rigid_body_set);

        Simulator {
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            rigid_body_set,
            collider_set,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            ball1_handle,
            ball2_handle,
        }
    }

    pub fn step(&mut self) -> (Vec<Vector<f32>>, u128) {
        let start = Instant::now();

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );

        let ball1_position = self.rigid_body_set[self.ball1_handle].translation().clone();
        let ball2_position = self.rigid_body_set[self.ball2_handle].translation().clone();

        let duration = start.elapsed();
        let time_taken_us = duration.as_micros();

        (vec![ball1_position, ball2_position], time_taken_us)
    }
}
