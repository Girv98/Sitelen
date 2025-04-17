use glam::Vec3;

use crate::vertex::Vertex;

// #[cfg(not(feature = "indexed"))]
pub fn get_sphere_vertices(
    // _index_offset: u32,
    // position: Vec3,
    radius: f32,
    // rotation: Vec3,
) -> (Vec<Vertex>, Vec<u32>) {
    let phi_len = core::f32::consts::PI * 2.0;
    let theta_len = core::f32::consts::PI;

    // let radius = 1.0;
    let h_segments = 500;
    let v_segments = 340;

    let mut index: u32 = 0;
    let mut grid: Vec<Vec<u32>> = vec![];

    let mut vertices = vec![];
    let mut indices: Vec<u32> = vec![];
    for iy in 0..=v_segments {
        let mut vertices_row: Vec<u32> = vec![];
        let v = iy as f32 / v_segments as f32;
        // poles
        let mut u_offset = 0.0;
        if iy == 0 {
            u_offset = 0.5 / h_segments as f32;
        } else if iy == h_segments {
            u_offset = -0.5 / h_segments as f32;
        }

        for ix in 0..=h_segments {
            let u = ix as f32 / h_segments as f32;

            // vertex
            let position = [
                radius * (u * phi_len).cos() * (v * theta_len).sin(),
                -radius * (v * theta_len).cos(),
                -radius * (u * phi_len).sin() * (v * theta_len).sin(),
            ];
            // normal
            let normal = Vec3::new(position[0], position[1], position[2])
                .normalize()
                .to_array();
            // uv
            let uv = [u + u_offset, 1.0 - v];
            vertices.push(Vertex { position, tex_coords: uv, normal });

            vertices_row.push(index);
            index += 1;
        }
        grid.push(vertices_row);
    }

    // indices
    for iy in 0..v_segments {
        for ix in 0..h_segments {
            let a = grid[iy][ix + 1];
            let b = grid[iy][ix];
            let c = grid[iy + 1][ix];
            let d = grid[iy + 1][ix + 1];

            if iy != 0 {
                indices.push(a);
                indices.push(b);
                indices.push(d);
            }
            // pole
            if iy != v_segments - 1 {
                indices.push(b);
                indices.push(c);
                indices.push(d);
            }
        }
    }

    println!("Verts: {}", vertices.len());
    println!("Indcs: {}", indices.len());
    indices.reverse();
    (vertices, indices)
}