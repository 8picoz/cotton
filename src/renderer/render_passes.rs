use ash::Device;
use ash::vk::{AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, Extent2D, Format, Framebuffer, FramebufferCreateInfo, ImageLayout, ImageView, PipelineBindPoint, PipelineStageFlags, RenderPass, RenderPassCreateInfo, SampleCountFlags, SUBPASS_EXTERNAL, SubpassDependency, SubpassDescription};
use log::info;

use crate::renderer::backends::Backends;

pub struct RenderPasses<'a> {
    device: &'a Device,
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
}

impl<'a> RenderPasses<'a> {
    pub fn new(
        backends: &'a Backends,
        format: Format,
        image_views: Vec<ImageView>,
        extent: Extent2D,
    ) -> Self {
        let color_attachment = AttachmentDescription::builder()
            .format(format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = AttachmentReference::builder()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = SubpassDescription::builder()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .build();

        let dependency = SubpassDependency::builder()
            .src_subpass(SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(AccessFlags::empty())
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let render_pass_info = RenderPassCreateInfo::builder()
            .attachments(&[color_attachment])
            .subpasses(&[subpass])
            .dependencies(&[dependency])
            .build();

        let render_pass =
            unsafe {
                backends.device.create_render_pass(&render_pass_info, None).unwrap()
            };

        let mut framebuffers = vec![];

        for image_view in image_views {
            let framebuffer_info = FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&[image_view])
                .width(extent.width)
                .height(extent.height)
                .layers(1)
                .build();

            framebuffers.push(
                unsafe {
                    backends.device.create_framebuffer(&framebuffer_info, None).unwrap()
                }
            );
        }

        info!("Create Render passes");

        Self {
            device: &backends.device,
            render_pass,
            framebuffers,
        }
    }
}

impl Drop for RenderPasses<'_> {
    fn drop(&mut self) {
        unsafe {
            for framebuffer in self.framebuffers.clone() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}