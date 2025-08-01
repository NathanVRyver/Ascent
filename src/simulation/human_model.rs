use bevy::prelude::*;

pub fn create_human_flyer_bundle(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> (Mesh3d, MeshMaterial3d<StandardMaterial>, Vec<(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform)>) {
    
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.7, 0.6), // Skin tone
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });
    
    let clothing_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.3, 0.8), // Blue clothing
        metallic: 0.0,
        perceptual_roughness: 0.9,
        ..default()
    });
    
    let hair_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.2, 0.1), // Brown hair
        metallic: 0.0,
        perceptual_roughness: 0.95,
        ..default()
    });
    
    // Main torso
    let torso = (
        Mesh3d(meshes.add(Capsule3d::new(0.25, 0.8))),
        MeshMaterial3d(clothing_material.clone()),
    );
    
    // Body parts as children
    let mut body_parts = Vec::new();
    
    // Head
    body_parts.push((
        Mesh3d(meshes.add(Sphere::new(0.12))),
        MeshMaterial3d(body_material.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.65, 0.0)),
    ));
    
    // Hair
    body_parts.push((
        Mesh3d(meshes.add(Sphere::new(0.13))),
        MeshMaterial3d(hair_material),
        Transform::from_translation(Vec3::new(0.0, 0.72, 0.0)),
    ));
    
    // Arms
    body_parts.push((
        Mesh3d(meshes.add(Capsule3d::new(0.06, 0.5))),
        MeshMaterial3d(body_material.clone()),
        Transform::from_translation(Vec3::new(-0.35, 0.25, 0.0))
            .with_rotation(Quat::from_rotation_z(0.3)),
    ));
    
    body_parts.push((
        Mesh3d(meshes.add(Capsule3d::new(0.06, 0.5))),
        MeshMaterial3d(body_material.clone()),
        Transform::from_translation(Vec3::new(0.35, 0.25, 0.0))
            .with_rotation(Quat::from_rotation_z(-0.3)),
    ));
    
    // Legs
    body_parts.push((
        Mesh3d(meshes.add(Capsule3d::new(0.08, 0.7))),
        MeshMaterial3d(clothing_material.clone()),
        Transform::from_translation(Vec3::new(-0.12, -0.75, 0.0)),
    ));
    
    body_parts.push((
        Mesh3d(meshes.add(Capsule3d::new(0.08, 0.7))),
        MeshMaterial3d(clothing_material.clone()),
        Transform::from_translation(Vec3::new(0.12, -0.75, 0.0)),
    ));
    
    // Feet
    body_parts.push((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.08, 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1), // Black shoes
            metallic: 0.2,
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_translation(Vec3::new(-0.12, -1.15, 0.1)),
    ));
    
    body_parts.push((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.08, 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1), // Black shoes  
            metallic: 0.2,
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.12, -1.15, 0.1)),
    ));
    
    (torso.0, torso.1, body_parts)
}

pub fn create_realistic_wings(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> (Mesh3d, MeshMaterial3d<StandardMaterial>) {
    
    let wing_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.9, 0.95, 0.85),
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        metallic: 0.1,
        perceptual_roughness: 0.2,
        ..default()
    });
    
    // Create a wing shape using multiple connected quads to simulate wing membrane
    let wing_mesh = create_wing_membrane_mesh();
    
    (
        Mesh3d(meshes.add(wing_mesh)),
        MeshMaterial3d(wing_material),
    )
}

fn create_wing_membrane_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Wing dimensions
    let wing_span = 5.0;
    let root_chord = 1.2;
    let tip_chord = 0.4;
    let segments = 10;
    
    // Generate wing surface
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let y_pos = wing_span * t - wing_span * 0.5;
        let chord = root_chord + (tip_chord - root_chord) * t;
        let sweep = t * 0.5; // Add some sweep
        
        // Leading edge
        positions.push([sweep, y_pos, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.0, t]);
        
        // Trailing edge  
        positions.push([sweep + chord, y_pos, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([1.0, t]);
        
        // Create triangles
        if i < segments {
            let base = i * 2;
            
            // First triangle
            indices.push(base as u32);
            indices.push((base + 1) as u32);
            indices.push((base + 2) as u32);
            
            // Second triangle
            indices.push((base + 1) as u32);
            indices.push((base + 3) as u32);
            indices.push((base + 2) as u32);
        }
    }
    
    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}