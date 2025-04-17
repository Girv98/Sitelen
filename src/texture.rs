use std::{fs::File, io::BufReader};

use image::{GenericImageView, ImageBuffer, ImageError, ImageReader, Rgb, Rgba};

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// Creates a new texture from a byte array.
    ///
    /// Arguments:
    ///
    /// * `device`: The wgpu device for which the texture will be generated.
    /// * `queue`: The wgpu queue for which the texture will be generated.
    /// * `bytes`: The byte array containing the image / texture.
    /// * `label`: The label of the new texture.
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, ImageError> {
        let file = File::open("./src/assets/test.png")?;
        let buf = BufReader::new(file);
        let mut reader = ImageReader::new(buf);
        reader.no_limits();
        let image = reader.with_guessed_format()?.decode()?;
        // let image = ImageReader::open("./src/assets/test.png")?.decode()?;
        // let image = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, &image, Some(label)))
    }

    // fn rgba8_to_rgb8(input: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    //     let width = input.width() as usize;
    //     let height = input.height() as usize;
        
    //     // Get the raw image data as a vector
    //     let input: &Vec<u8> = input.as_raw();
        
    //     // Allocate a new buffer for the RGB image, 3 bytes per pixel
    //     let mut output_data = vec![0u8; width * height * 3];
        
    //     let mut i = 0;
    //     // Iterate through 4-byte chunks of the image data (RGBA bytes)
    //     for chunk in input.chunks(4) {
    //         // ... and copy each of them to output, leaving out the A byte
    //         output_data[i..i+3].copy_from_slice(&chunk[0..3]);
    //         i+=3;
    //     }
        
    //     // Construct a new image
    //     ImageBuffer::from_raw(width as u32, height as u32, output_data).unwrap()
    // }

    // fn rgb8_to_rgba8(input: ImageBuffer<Rgb<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    //     let width = input.width() as usize;
    //     let height = input.height() as usize;
        
    //     // Get the raw image data as a vector
    //     let input: &Vec<u8> = input.as_raw();
        
    //     // Allocate a new buffer for the RGBA image, 4 bytes per pixel
    //     let mut output_data = vec![0u8; width * height * 4];
        
    //     let mut i = 0;
    //     // Iterate through 3-byte chunks of the image data (RGBA bytes)
    //     for chunk in input.chunks(3) {
    //         // ... and copy each of them to output, leaving out the A byte
    //         output_data[i..i+3].copy_from_slice(&chunk[0..3]);
    //         output_data[i+3] = 0;
    //         i+=4;
    //     }
        
    //     // Construct a new image
    //     ImageBuffer::from_raw(width as u32, height as u32, output_data).unwrap()
    // }


    /// Creates a new texture from a [image::DynamicImage].
    ///
    /// Arguments:
    ///
    /// * `device`: The wgpu device for which the texture will be generated.
    /// * `queue`: The wgpu queue for which the texture will be generated.
    /// * `image`: The source [image::DynamicImage] of the new texture.
    /// * `label`: The label of the new texture.
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
        label: Option<&str>,
    ) -> Self {
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    /// Creates a new depth texture.
    ///
    /// Arguments:
    ///
    /// * `device`: The wgpu device for which the texture will be generated.
    /// * `config`: The wgpu surface configuration for which the texture will be generated.
    /// * `sample_count`: This has to be the same as the number of samples used for _MSAA_.
    /// * `label`: The label of the texture.
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
        label: &str,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, /*  | wgpu::TextureUsages::TEXTURE_BINDING */
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    /// Creates a framebuffer that can be used for multisample anti-aliasing.
    ///
    /// Arguments:
    ///
    /// * `device`: The wgpu device for which the texture will be generated.
    /// * `config`: The wgpu surface configuration for which the texture will be generated.
    /// * `sample_count`: The sample count used for _MSAA_. Valid values are `1` (no MSAA) or `4`.
    /// * `label`: The label of the texture.
    pub fn create_multisampled_framebuffer(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
        label: &str,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some(label),
            view_formats: &[],
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
