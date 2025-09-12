use pelican_ui::{Component, Context};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::hardware::Camera;
use pelican_ui_std::{Size, Padding, Offset};

#[derive(Debug)]
pub enum SpaceEvenly {
    Vertical(Offset, Size, Padding),
    Horizontal(Offset, Size, Padding),
}

impl SpaceEvenly {
    pub fn vertical(offset: Offset, size: Size, padding: Padding) -> Self {
        SpaceEvenly::Vertical(offset, size, padding)
    }

    pub fn horizontal(offset: Offset, size: Size, padding: Padding) -> Self {
        SpaceEvenly::Horizontal(offset, size, padding)
    }
}

impl Layout for SpaceEvenly {
    fn request_size(&self, _ctx: &mut Context, children: Vec<SizeRequest>) -> SizeRequest {
        let (widths, heights): (Vec<_>, Vec<_>) = children.iter()
            .map(|i| ((i.min_width(), i.max_width()), (i.min_height(), i.max_height())))
            .unzip();

        match self {
            SpaceEvenly::Horizontal(_, size, padding)
            | SpaceEvenly::Vertical(_, size, padding) => {
                let w = size.get(widths, Size::max);
                let h = size.get(heights, Size::max);
                padding.adjust_request(SizeRequest::new(w.0, h.0, w.1, h.1))
            }
        }
    }

    fn build(&self, _ctx: &mut Context, parent: (f32, f32), children: Vec<SizeRequest>) -> Vec<Area> {
        match self {
            SpaceEvenly::Horizontal(offset, _, padding) => {
                let parent = padding.adjust_size(parent);
                let n = children.len();
                let gap = parent.0 / (n + 1) as f32;

                children.into_iter().enumerate().map(|(i, req)| {
                    let size = req.get((gap, parent.1));
                    let x = (i as f32 + 1.0) * gap - size.0 / 2.0;
                    let y = padding.adjust_offset((offset.get(parent.0, size.0), 0.0)).1;
                    Area { offset: (x, y), size }
                }).collect()
            }

            SpaceEvenly::Vertical(offset, _, padding) => {
                let parent = padding.adjust_size(parent);
                let n = children.len();
                let gap = parent.1 / (n + 1) as f32;

                children.into_iter().enumerate().map(|(i, req)| {
                    let size = req.get((parent.0, gap));
                    let x = padding.adjust_offset((offset.get(parent.0, size.0), 0.0)).0;
                    let y = (i as f32 + 1.0) * gap - size.1 / 2.0;
                    Area { offset: (x, y), size }
                }).collect()
            }
        }
    }
}