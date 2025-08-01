use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct WingGeometry {
    pub root_chord: f32,
    pub tip_chord: f32,
    pub span: f32,
    pub sweep_angle: f32,
    pub dihedral_angle: f32,
    pub segments: u32,
}

impl Default for WingGeometry {
    fn default() -> Self {
        Self {
            root_chord: 1.5,
            tip_chord: 0.8,
            span: 10.0,
            sweep_angle: 5.0_f32.to_radians(),
            dihedral_angle: 5.0_f32.to_radians(),
            segments: 20,
        }
    }
}

pub fn create_wing_mesh(geometry: &WingGeometry) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    let half_span = geometry.span / 2.0;
    let segments = geometry.segments as f32;
    
    for i in 0..=geometry.segments {
        let t = i as f32 / segments;
        let span_position = -half_span + geometry.span * t;
        
        let chord = geometry.root_chord + (geometry.tip_chord - geometry.root_chord) * t;
        
        let sweep_offset = span_position.abs() * geometry.sweep_angle.tan();
        let dihedral_height = span_position.abs() * geometry.dihedral_angle.tan();
        
        let leading_edge = Vec3::new(sweep_offset, dihedral_height, span_position);
        let trailing_edge = Vec3::new(sweep_offset + chord, dihedral_height, span_position);
        
        let profile_points = generate_airfoil_profile(chord, 10);
        
        for (j, point) in profile_points.iter().enumerate() {
            let position = Vec3::new(
                sweep_offset + point.x,
                dihedral_height + point.y,
                span_position,
            );
            
            positions.push([position.x, position.y, position.z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([t, j as f32 / (profile_points.len() - 1) as f32]);
        }
    }
    
    let profile_count = 10;
    for i in 0..geometry.segments {
        for j in 0..profile_count - 1 {
            let base = i * profile_count + j;
            let next_span = base + profile_count;
            
            indices.push(base as u32);
            indices.push((base + 1) as u32);
            indices.push(next_span as u32);
            
            indices.push((base + 1) as u32);
            indices.push((next_span + 1) as u32);
            indices.push(next_span as u32);
        }
    }
    
    Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}

fn generate_airfoil_profile(chord: f32, points: usize) -> Vec<Vec2> {
    let mut profile = Vec::new();
    
    for i in 0..points {
        let t = i as f32 / (points - 1) as f32;
        let x = chord * t;
        
        let thickness = 0.12;
        let camber = 0.04;
        let max_camber_pos = 0.4;
        
        let thickness_y = 5.0 * thickness * chord * (
            0.2969 * (x / chord).sqrt()
            - 0.1260 * (x / chord)
            - 0.3516 * (x / chord).powi(2)
            + 0.2843 * (x / chord).powi(3)
            - 0.1015 * (x / chord).powi(4)
        );
        
        let camber_y = if x < max_camber_pos * chord {
            camber * chord * (2.0 * max_camber_pos * (x / chord) - (x / chord).powi(2)) / max_camber_pos.powi(2)
        } else {
            camber * chord * (1.0 - 2.0 * max_camber_pos + 2.0 * max_camber_pos * (x / chord) - (x / chord).powi(2)) / (1.0 - max_camber_pos).powi(2)
        };
        
        let y = camber_y + thickness_y * (1.0 - 2.0 * (i % 2) as f32);
        profile.push(Vec2::new(x, y));
    }
    
    profile
}

pub fn create_flapping_wing_animation(
    time: f32,
    flap_frequency: f32,
    flap_amplitude: f32,
) -> Quat {
    let flap_angle = (time * flap_frequency * 2.0 * std::f32::consts::PI).sin() * flap_amplitude;
    Quat::from_rotation_x(flap_angle)
}