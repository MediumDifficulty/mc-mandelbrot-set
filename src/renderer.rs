use std::sync::Arc;

use cudarc::driver::CudaDevice;
use cudarc::driver::CudaSlice;
use cudarc::driver::DriverError;
use cudarc::driver::LaunchAsync;
use cudarc::driver::LaunchConfig;
use cudarc::nvrtc;
use valence::glam::uvec2;

const BLACK_ID: u8 = 119;

const CUDA_SRC: &str = include_str!("./assets/renderer.cu");
const BAKED_MAP_IDS: &[u8] = include_bytes!("./assets/closest_colours.dat");

pub struct Renderer {
    pub width: u32,
    pub height: u32,
    pub iterations: u32,
    device: Arc<CudaDevice>,
    dev_image: CudaSlice<u8>,
    cached_values: CudaSlice<u8>,
}

impl Renderer {
    pub fn new(width: u32, height: u32, iterations: u32) -> Result<Self, DriverError> {
        let device = CudaDevice::new(0).unwrap();

        device.load_ptx(nvrtc::compile_ptx(CUDA_SRC).unwrap(), "renderer", &["render_kernel"])?;

        let dev_image = device.alloc_zeros::<u8>((width * height) as usize)?;
        let cached_values = device.htod_sync_copy(BAKED_MAP_IDS)?;

        Ok(Renderer {
            width,
            height,
            device,
            cached_values,
            dev_image,
            iterations,
        })
    }

    pub fn render(&mut self) -> Result<Vec<[u8; 16384]>, DriverError> {
        // info!("a");
        let launch_config = LaunchConfig::for_num_elems(self.width * self.height);
        let render_func = self.device.get_func("renderer", "render_kernel").unwrap();
        unsafe { render_func.launch(launch_config, (&mut self.dev_image, self.width, self.height, self.iterations, &self.cached_values, self.width * self.height)) }?;
        // info!("b");
        let rendered = self.device.dtoh_sync_copy(&self.dev_image)?;
        // info!("c");
        Ok(self.plot(&rendered))
    }

    fn plot(&self, pixels: &[u8]) -> Vec<[u8; 16384]> {
        let width_maps = num_integer::div_ceil(self.width, 128);
        let height_maps = num_integer::div_ceil(self.height, 128);

        let mut maps = vec![[BLACK_ID; 16384]; (width_maps * height_maps) as usize];
        
        let offset = uvec2((width_maps * 128 - self.width) / 2, (height_maps * 128 - self.height) / 2);

        for (i, &colour) in pixels.iter().enumerate() {
            let coords = uvec2(i as u32 % self.width, i as u32 / self.width) + offset;
            let block_coords = coords / 128;
            let map_coords = coords % 128;

            maps[(block_coords.y * width_maps + block_coords.x) as usize][(map_coords.y * 128 + map_coords.x) as usize] = colour;
        }

        maps
    }
}