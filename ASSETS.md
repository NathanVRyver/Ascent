# Assets Guide

This project supports loading 3D models and textures to make the simulation more realistic.

## Required Assets

Place these files in the specified folders:

### Models (`.obj` format)
- `assets/models/human.obj` - Human character model
- `assets/models/wings.obj` - Dragonfly-style wings

### Textures (`.png` format)  
- `assets/textures/human.png` - Human skin/clothing texture
- `assets/textures/wings.png` - Wing membrane texture
- `assets/textures/terrain.png` - Terrain texture

## Where to Find Free Assets

### 3D Models
1. **Mixamo** (https://www.mixamo.com)
   - Free 3D characters (requires Adobe account)
   - Download as FBX, convert to OBJ using Blender

2. **Sketchfab** (https://sketchfab.com)
   - Search for "human character" or "person"
   - Filter by "Downloadable" and "Free"
   - Many CC0 (public domain) models available

3. **TurboSquid** (https://www.turbosquid.com/Search/3D-Models/free)
   - Free 3D models section
   - Good quality human models

4. **CGTrader** (https://www.cgtrader.com/free-3d-models)
   - Free 3D model section

### Textures
1. **Textures.com** (formerly CGTextures)
   - Free textures (requires account)
   - Great for terrain, skin, fabric textures

2. **Poly Haven** (https://polyhaven.com)
   - CC0 textures
   - High quality, no account needed

3. **Freepik** (https://www.freepik.com)
   - Free textures (attribution required)

## Model Requirements

### Human Model
- Should be in a T-pose or neutral standing position
- Preferably low-poly (under 5000 triangles for performance)
- Size should be roughly 1.8 units tall (will be scaled automatically)

### Wings Model
- Dragonfly or fairy-style wings
- 4 separate wings or wing pairs
- Translucent/thin design works best

## Texture Requirements

- **Resolution**: 512x512 or 1024x1024 pixels
- **Format**: PNG with transparency support
- **Style**: Realistic or stylized to match your preference

## Converting Models

If you have FBX or other formats, use **Blender** (free) to convert:

1. Import your model (File > Import)
2. Export as OBJ (File > Export > Wavefront OBJ)
3. Make sure to export with normals and UVs

## File Structure
```
Ascent/
├── assets/
│   ├── models/
│   │   ├── human.obj
│   │   └── wings.obj
│   └── textures/
│       ├── human.png
│       ├── wings.png
│       └── terrain.png
└── src/
    └── ...
```

## Notes

- The system will fall back to procedural graphics if assets aren't found
- Start with just textures - they'll make the biggest visual difference
- Models are more complex to implement but will be loaded automatically once placed