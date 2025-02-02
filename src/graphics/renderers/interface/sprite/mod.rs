// TODO: remove once no longer needed
#[allow(clippy::needless_question_mark)]
mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/renderers/interface/sprite/vertex_shader.glsl"
    }
}

// TODO: remove once no longer needed
#[allow(clippy::needless_question_mark)]
mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/graphics/renderers/interface/sprite/fragment_shader.glsl"
    }
}

use std::iter;
use std::sync::Arc;

use cgmath::{Vector2, Vector4};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, DeviceOwned};
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::Subpass;
use vulkano::sampler::{Filter, Sampler, SamplerCreateInfo};
use vulkano::shader::ShaderModule;

use self::vertex_shader::ty::Constants;
use crate::graphics::*;

unsafe impl bytemuck::Zeroable for Constants {}
unsafe impl bytemuck::Pod for Constants {}

pub struct SpriteRenderer {
    memory_allocator: Arc<MemoryAllocator>,
    pipeline: Arc<GraphicsPipeline>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    nearest_sampler: Arc<Sampler>,
    linear_sampler: Arc<Sampler>,
}

impl SpriteRenderer {
    pub fn new(memory_allocator: Arc<MemoryAllocator>, subpass: Subpass, viewport: Viewport) -> Self {
        let device = memory_allocator.device().clone();
        let vertex_shader = vertex_shader::load(device.clone()).unwrap();
        let fragment_shader = fragment_shader::load(device.clone()).unwrap();
        let pipeline = Self::create_pipeline(device.clone(), subpass, viewport, &vertex_shader, &fragment_shader);

        let nearest_sampler = Sampler::new(device.clone(), SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            ..Default::default()
        })
        .unwrap();

        let linear_sampler = Sampler::new(device, SamplerCreateInfo::simple_repeat_linear_no_mipmap()).unwrap();

        Self {
            memory_allocator,
            pipeline,
            vertex_shader,
            fragment_shader,
            nearest_sampler,
            linear_sampler,
        }
    }

    pub fn recreate_pipeline(&mut self, device: Arc<Device>, subpass: Subpass, viewport: Viewport) {
        self.pipeline = Self::create_pipeline(device, subpass, viewport, &self.vertex_shader, &self.fragment_shader);
    }

    fn create_pipeline(
        device: Arc<Device>,
        subpass: Subpass,
        viewport: Viewport,
        vertex_shader: &ShaderModule,
        fragment_shader: &ShaderModule,
    ) -> Arc<GraphicsPipeline> {
        GraphicsPipeline::start()
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant(iter::once(viewport)))
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .color_blend_state(ColorBlendState::new(1).blend(INTERFACE_ATTACHMENT_BLEND))
            .multisample_state(MultisampleState {
                rasterization_samples: vulkano::image::SampleCount::Sample4,
                ..Default::default()
            })
            .render_pass(subpass)
            .build(device)
            .unwrap()
    }

    fn build(
        &self,
        render_target: &mut <InterfaceRenderer as Renderer>::Target,
        texture: Texture,
        screen_position: Vector2<f32>,
        screen_size: Vector2<f32>,
        clip_size: Vector4<f32>,
        texture_position: Vector2<f32>,
        texture_size: Vector2<f32>,
        color: Color,
        smooth: bool,
    ) {
        let layout = self.pipeline.layout().clone();
        let descriptor_layout = layout.set_layouts().get(0).unwrap().clone();

        let sampler = match smooth {
            true => self.linear_sampler.clone(),
            false => self.nearest_sampler.clone(),
        };

        let set = PersistentDescriptorSet::new(&*self.memory_allocator, descriptor_layout, [
            WriteDescriptorSet::image_view_sampler(0, texture, sampler),
        ])
        .unwrap();

        let constants = Constants {
            screen_position: screen_position.into(),
            screen_size: screen_size.into(),
            clip_size: clip_size.into(),
            texture_position: texture_position.into(),
            texture_size: texture_size.into(),
            color: [color.red_f32(), color.green_f32(), color.blue_f32(), color.alpha_f32()],
        };

        render_target
            .state
            .get_builder()
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(PipelineBindPoint::Graphics, layout.clone(), 0, set)
            .push_constants(layout, 0, constants)
            .draw(6, 1, 0, 0)
            .unwrap();
    }

    pub fn render(
        &self,
        render_target: &mut <InterfaceRenderer as Renderer>::Target,
        texture: Texture,
        window_size: Vector2<usize>,
        screen_position: Vector2<f32>,
        screen_size: Vector2<f32>,
        clip_size: Vector4<f32>,
        color: Color,
        smooth: bool,
    ) {
        let half_screen = Vector2::new(window_size.x as f32 / 2.0, window_size.y as f32 / 2.0);
        let screen_position = Vector2::new(screen_position.x / half_screen.x, screen_position.y / half_screen.y);
        let screen_size = Vector2::new(screen_size.x / half_screen.x, screen_size.y / half_screen.y);

        // TODO: don't invoke build. inline code
        self.build(
            render_target,
            texture,
            screen_position,
            screen_size,
            clip_size,
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 1.0),
            color,
            smooth,
        );
    }
}
