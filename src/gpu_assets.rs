use crate::assets::{AssetEvent, Assets, Handle};

pub trait GpuAsset: Sized {
    type Gpu;

    fn upload(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Self::Gpu;

    fn reupload(&self, gpu: &mut Self::Gpu, device: &wgpu::Device, queue: &wgpu::Queue) {
        *gpu = self.upload(device, queue);
    }
}

struct Entry<G> {
    generation: u32,
    gpu: G,
}

pub struct GpuAssets<A: GpuAsset> {
    entries: Vec<Option<Entry<A::Gpu>>>,
}

impl<A: GpuAsset> Default for GpuAssets<A> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<A: GpuAsset> GpuAssets<A> {
    pub fn sync(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, assets: &mut Assets<A>) {
        for event in assets.take_events() {
            match event {
                AssetEvent::Changed(handle) => {
                    let Some(asset) = assets.get(handle) else {
                        continue;
                    };

                    if self.entries.len() <= handle.index() {
                        self.entries.resize_with(handle.index() + 1, || None);
                    }

                    let slot = &mut self.entries[handle.index()];
                    if let Some(entry) = slot
                        && entry.generation == handle.generation()
                    {
                        asset.reupload(&mut entry.gpu, device, queue);
                    } else {
                        *slot = Some(Entry {
                            generation: handle.generation(),
                            gpu: asset.upload(device, queue),
                        });
                    }
                }
                AssetEvent::Removed(handle) => {
                    if let Some(slot) = self.entries.get_mut(handle.index())
                        && slot
                            .as_ref()
                            .is_some_and(|entry| entry.generation == handle.generation())
                    {
                        *slot = None;
                    }
                }
            }
        }
    }

    pub fn get(&self, handle: Handle<A>) -> Option<&A::Gpu> {
        self.entries
            .get(handle.index())?
            .as_ref()
            .filter(|entry| entry.generation == handle.generation())
            .map(|entry| &entry.gpu)
    }
}
