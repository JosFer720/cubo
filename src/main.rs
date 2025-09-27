use minifb::{Key, Window, WindowOptions};
use nalgebra::{Point3, Vector3};
use image::{RgbImage, Rgb};
use rayon::prelude::*;
use std::fs;

const WIDTH: usize = 600;
const HEIGHT: usize = 450;

// Definición de tipos de bloques
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum BlockType {
    Aire = 0,
    TerracotaNaranja = 1,    // n
    Netherrack = 2,          // i
    BloqueMagma = 3,         // l
    TerracotaNormal = 4,     // t
    TerracotaAmarilla = 5,   // a
    ObsidianaNormal = 6,     // o
    Lava = 7,                // p
    BloqueOro = 8,           // y
    Cofre = 9,               // c
    EscaleraPiedra = 10,     // e
    SlabPiedra = 11,         // s
    CryingObsidian = 12,     // k
}

// Arrays estáticos para lookup rápido de colores fallback
static BLOCK_COLORS: [(f32, f32, f32); 13] = [
    (0.0, 0.0, 0.0),      // Aire
    (0.8, 0.4, 0.1),      // Terracota naranja
    (0.4, 0.2, 0.2),      // Netherrack
    (0.9, 0.3, 0.1),      // Bloque magma
    (0.6, 0.4, 0.3),      // Terracota normal
    (0.9, 0.8, 0.2),      // Terracota amarilla
    (0.1, 0.1, 0.2),      // Obsidiana
    (1.0, 0.5, 0.0),      // Lava
    (1.0, 0.8, 0.0),      // Oro
    (0.5, 0.3, 0.2),      // Cofre
    (0.5, 0.5, 0.5),      // Escalera piedra
    (0.6, 0.6, 0.6),      // Slab piedra
    (0.3, 0.1, 0.4),      // Crying obsidian
];

static EMISSIVE_BLOCKS: [bool; 13] = [
    false, false, false, true,  // 0-3 (magma emite luz)
    false, false, false, true,  // 4-7 (lava emite luz)
    false, false, false, false, false, // 8-12
];

// Estructura para manejar texturas
pub struct Texture {
    pixels: Vec<Rgb<u8>>,
    width: u32,
    height: u32,
}

impl Texture {
    fn new(width: u32, height: u32, color: Rgb<u8>) -> Self {
        Texture {
            pixels: vec![color; (width * height) as usize],
            width,
            height,
        }
    }
    
    fn from_image(img: RgbImage) -> Self {
        let (width, height) = (img.width(), img.height());
        let pixels = img.into_raw()
            .chunks(3)
            .map(|chunk| Rgb([chunk[0], chunk[1], chunk[2]]))
            .collect();
        
        Texture { pixels, width, height }
    }
    
    // Obtener color en coordenadas UV (0.0 a 1.0)
    fn sample(&self, u: f32, v: f32) -> Rgb<u8> {
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        let index = (y * self.width + x) as usize;
        self.pixels[index]
    }
}

pub struct TextureManager {
    textures: Vec<Texture>,
}

impl TextureManager {
    fn new() -> Self {
        TextureManager {
            textures: Vec::new(),
        }
    }
    
    fn load_textures(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Nombres de archivos de texturas para cada tipo de bloque
        let texture_files = [
            "aire.png",              // 0 - Aire (no se usa)
            "terracota_naranja.png", // 1 - Terracota naranja
            "netherrack.png",        // 2 - Netherrack
            "bloque_magma.png",      // 3 - Bloque magma
            "terracota_normal.png",  // 4 - Terracota normal
            "terracota_amarilla.png",// 5 - Terracota amarilla
            "obsidiana.png",         // 6 - Obsidiana normal
            "lava.png",              // 7 - Lava
            "bloque_oro.png",        // 8 - Bloque oro
            "cofre.png",             // 9 - Cofre
            "piedra.png",            // 10 - Escalera piedra (usa textura de piedra)
            "piedra.png",            // 11 - Slab piedra (usa textura de piedra)
            "crying_obsidian.png",   // 12 - Crying obsidian
        ];
        
        // Asegurar que la carpeta 'textures' exista
        let _ = std::fs::create_dir_all("textures");

        for (i, filename) in texture_files.iter().enumerate() {
            let path = format!("textures/{}", filename);

            // Si no existe el archivo, crear un PNG placeholder con el color base
            if !std::path::Path::new(&path).exists() {
                let fallback_color = BLOCK_COLORS[i];
                let mut img = RgbImage::new(16, 16);
                let px = image::Rgb([
                    (fallback_color.0 * 255.0) as u8,
                    (fallback_color.1 * 255.0) as u8,
                    (fallback_color.2 * 255.0) as u8,
                ]);
                for y in 0..16 {
                    for x in 0..16 {
                        img.put_pixel(x, y, px);
                    }
                }
                // Intentar guardar; si falla, seguiremos con el fallback en memoria
                if let Err(e) = img.save(&path) {
                    println!("⚠ No se pudo crear placeholder {}: {}", path, e);
                } else {
                    println!("ℹ Placeholder creado: {}", path);
                }
            }

            match image::open(&path) {
                Ok(img) => {
                    let rgb_img = img.to_rgb8();
                    self.textures.push(Texture::from_image(rgb_img));
                    if i == 10 || i == 11 {
                        println!("✓ Textura cargada: {} (compartida para escalera/slab)", path);
                    } else {
                        println!("✓ Textura cargada: {}", path);
                    }
                }
                Err(_) => {
                    // Crear textura de color sólido como fallback en memoria
                    let fallback_color = BLOCK_COLORS[i];
                    let color = Rgb([
                        (fallback_color.0 * 255.0) as u8,
                        (fallback_color.1 * 255.0) as u8,
                        (fallback_color.2 * 255.0) as u8,
                    ]);
                    self.textures.push(Texture::new(16, 16, color));
                    println!("⚠ No se encontró {} y no se pudo cargar, usando color sólido", path);
                }
            }
        }
        
        Ok(())
    }
    
    fn get_texture(&self, block_type: BlockType) -> &Texture {
        &self.textures[block_type as usize]
    }
}

#[allow(dead_code)]
impl BlockType {
    #[inline]
    fn from_char(c: char) -> Self {
        match c {
            'n' => BlockType::TerracotaNaranja,
            'i' => BlockType::Netherrack,
            'l' => BlockType::BloqueMagma,
            't' => BlockType::TerracotaNormal,
            'a' => BlockType::TerracotaAmarilla,
            'o' => BlockType::ObsidianaNormal,
            'p' => BlockType::Lava,
            'y' => BlockType::BloqueOro,
            'c' => BlockType::Cofre,
            'e' => BlockType::EscaleraPiedra,
            's' => BlockType::SlabPiedra,
            'k' => BlockType::CryingObsidian,
            ' ' => BlockType::Aire,
            _ => BlockType::Aire,
        }
    }
    
    #[inline]
    fn get_color(self) -> (f32, f32, f32) {
        BLOCK_COLORS[self as usize]
    }
    
    #[inline]
    fn is_solid(self) -> bool {
        self != BlockType::Aire
    }
    
    #[inline]
    fn emits_light(self) -> bool {
        EMISSIVE_BLOCKS[self as usize]
    }
    
    #[inline]
    fn is_slab(self) -> bool {
        self == BlockType::SlabPiedra
    }
    
    #[inline]
    fn is_stairs(self) -> bool {
        self == BlockType::EscaleraPiedra
    }
}

// Ray / AABB intersection (slab method). Returns (tmin, tmax) if intersects.
fn ray_aabb_intersect(orig: &Point3<f32>, dir: &Vector3<f32>, min: &Point3<f32>, max: &Point3<f32>) -> Option<(f32,f32)> {
    // Use safe handling for zero components by treating them as infinite slabs
    let mut tmin = (min.x - orig.x) / dir.x;
    let mut tmax = (max.x - orig.x) / dir.x;
    if dir.x == 0.0 { tmin = f32::NEG_INFINITY; tmax = f32::INFINITY; }
    if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

    let mut tymin = (min.y - orig.y) / dir.y;
    let mut tymax = (max.y - orig.y) / dir.y;
    if dir.y == 0.0 { tymin = f32::NEG_INFINITY; tymax = f32::INFINITY; }
    if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

    if (tmin > tymax) || (tymin > tmax) { return None; }
    if tymin > tmin { tmin = tymin; }
    if tymax < tmax { tmax = tymax; }

    let mut tzmin = (min.z - orig.z) / dir.z;
    let mut tzmax = (max.z - orig.z) / dir.z;
    if dir.z == 0.0 { tzmin = f32::NEG_INFINITY; tzmax = f32::INFINITY; }
    if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

    if (tmin > tzmax) || (tzmin > tmax) { return None; }
    if tzmin > tmin { tmin = tzmin; }
    if tzmax < tmax { tmax = tzmax; }

    Some((tmin, tmax))
}

pub struct VoxelWorld {
    blocks: Vec<BlockType>,
    width: usize,
    height: usize,
    depth: usize,
}

impl VoxelWorld {
    fn new() -> Self {
        VoxelWorld {
            blocks: Vec::new(),
            width: 0,
            height: 0,
            depth: 0,
        }
    }
    
    fn load_from_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut layers = Vec::new();
        
        // Cargar archivos desde la carpeta "capas"
        for i in 0..12 {
            let filename = format!("capas/{}.txt", i);
            match fs::read_to_string(&filename) {
                Ok(content) => {
                    layers.push(content);
                    println!("Cargado: {}", filename);
                }
                Err(_) => {
                    println!("No se pudo cargar {}, usando capa vacía", filename);
                    layers.push("               \n".repeat(19));
                }
            }
        }
        
        self.load_from_layer_data(&layers);
        Ok(())
    }
    
    fn load_from_layer_data(&mut self, layer_data: &[String]) {
        if layer_data.is_empty() {
            println!("⚠️ No hay datos de capas para cargar");
            return;
        }
        
        self.width = 15;
        self.depth = 19;
        self.height = layer_data.len();
        
        self.blocks = vec![BlockType::Aire; self.width * self.height * self.depth];
        
        let mut blocks_loaded = 0;
        
        for (y, layer_str) in layer_data.iter().enumerate() {
            let lines: Vec<&str> = layer_str.lines().collect();
            println!("Capa {}: {} líneas", y, lines.len());
            
            for (z, line) in lines.iter().enumerate() {
                if z < self.depth {
                    let padded_line = format!("{:15}", line);
                    for (x, ch) in padded_line.chars().take(self.width).enumerate() {
                        let block_type = BlockType::from_char(ch);
                        if block_type.is_solid() {
                            blocks_loaded += 1;
                        }
                        let index = y * (self.width * self.depth) + z * self.width + x;
                        self.blocks[index] = block_type;
                    }
                }
            }
        }
        
        println!("Mundo cargado: {}x{}x{}", self.width, self.height, self.depth);
        println!("Total de bloques sólidos cargados: {}", blocks_loaded);
        
        // Mostrar algunos bloques de ejemplo para debugging
        println!("Ejemplo de bloques en capa 0:");
        for z in 0..3.min(self.depth) {
            for x in 0..5.min(self.width) {
                let block = self.get_block(x as i32, 0, z as i32);
                if block.is_solid() {
                    println!("  Bloque en ({}, 0, {}): {:?}", x, z, block);
                }
            }
        }
    }
    
    #[inline]
    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0 || y < 0 || z < 0 || 
           x >= self.width as i32 || y >= self.height as i32 || z >= self.depth as i32 {
            return BlockType::Aire;
        }
        let index = y as usize * (self.width * self.depth) + z as usize * self.width + x as usize;
        self.blocks[index]
    }
    
    fn check_special_collision(&self, x: i32, y: i32, z: i32, point: &Point3<f32>) -> bool {
        let block = self.get_block(x, y, z);
        // Local coordinates inside block
        let mut local_x = point.x - x as f32;
        let mut local_y = point.y - y as f32;
        let mut local_z = point.z - z as f32;

    // Use a slightly larger epsilon to be tolerant on boundaries (helps at distance and near emissive blocks)
    let eps = 1e-3;
        local_x = local_x.clamp(0.0 - eps, 1.0 + eps);
        local_y = local_y.clamp(0.0 - eps, 1.0 + eps);
        local_z = local_z.clamp(0.0 - eps, 1.0 + eps);

        match block {
            BlockType::SlabPiedra => local_y <= 0.5 + eps,
            BlockType::EscaleraPiedra => {
                if local_y <= 0.5 + eps {
                    true
                } else if local_y <= 1.0 + eps && local_z >= 0.5 - eps {
                    true
                } else {
                    false
                }
            },
            _ => {
                (local_x >= 0.0 - eps) && (local_x <= 1.0 + eps) &&
                (local_y >= 0.0 - eps) && (local_y <= 1.0 + eps) &&
                (local_z >= 0.0 - eps) && (local_z <= 1.0 + eps)
            }
        }
    }
    
    // Calcular coordenadas UV para mapeo de texturas con formas especiales
    fn calculate_uv(&self, hit_point: &Point3<f32>, normal: &Vector3<f32>, block_x: i32, block_y: i32, block_z: i32) -> (f32, f32) {
        let block_type = self.get_block(block_x, block_y, block_z);
        let local_x = hit_point.x - block_x as f32;
        let local_y = hit_point.y - block_y as f32;
        let local_z = hit_point.z - block_z as f32;
        // Helper closures for face-based mapping
        let map_face_x = |nx: f32| {
            // X-facing: u from Z, v from Y. Flip u for +X faces to keep orientation consistent.
            let u = if nx > 0.0 { 1.0 - local_z } else { local_z };
            let v = (1.0 - local_y).clamp(0.0, 1.0);
            (u.clamp(0.0, 1.0), v)
        };

        let map_face_y = |ny: f32| {
            // Y-facing (top/bottom): u from X, v from Z. Flip v for top faces for consistent orientation.
            let u = local_x.clamp(0.0, 1.0);
            let v = if ny > 0.0 { 1.0 - local_z } else { local_z };
            (u, v.clamp(0.0, 1.0))
        };

        let map_face_z = |nz: f32| {
            // Z-facing: u from X, v from Y. Flip u for -Z faces to keep orientation consistent.
            let u = if nz < 0.0 { 1.0 - local_x } else { local_x };
            let v = (1.0 - local_y).clamp(0.0, 1.0);
            (u.clamp(0.0, 1.0), v)
        };

        match block_type {
            BlockType::SlabPiedra => {
                // Slab occupies Y in [0, 0.5]. For side faces, scale Y into [0,1] by dividing by 0.5.
                if normal.y.abs() > 0.5 {
                    // Top/bottom of the slab: map full X/Z
                    map_face_y(normal.y)
                } else if normal.x.abs() > 0.5 {
                    // Side faces along X: u from Z, v from scaled Y (0..0.5 -> 0..1)
                    let u = if normal.x > 0.0 { 1.0 - local_z } else { local_z };
                    let v = (1.0 - (local_y / 0.5)).clamp(0.0, 1.0);
                    (u.clamp(0.0, 1.0), v)
                } else {
                    // Side faces along Z
                    let u = if normal.z < 0.0 { 1.0 - local_x } else { local_x };
                    let v = (1.0 - (local_y / 0.5)).clamp(0.0, 1.0);
                    (u.clamp(0.0, 1.0), v)
                }
            }
            BlockType::EscaleraPiedra => {
                // Stair shape: lower half (y <= 0.5) is base; upper half (y > 0.5 && z >= 0.5) is the step.
                if normal.y > 0.5 {
                    // Top face: distinguish between lower and upper step regions
                    if local_z >= 0.5 {
                        // Upper step top: map X across u, and map Z 0.5..1.0 -> 0..1
                        let u = local_x.clamp(0.0, 1.0);
                        let v = ((local_z - 0.5) / 0.5).clamp(0.0, 1.0);
                        (u, v)
                    } else {
                        // Base top: map X across u and Z 0.0..0.5 -> 0..1
                        let u = local_x.clamp(0.0, 1.0);
                        let v = (local_z / 0.5).clamp(0.0, 1.0);
                        (u, v)
                    }
                } else if normal.x.abs() > 0.5 {
                    // Side faces: if touching the upper step, map u to (local_z - 0.5)*2, v to local_y mapped into 0..1
                    if local_z >= 0.5 && local_y >= 0.5 {
                        let u = ((local_z - 0.5) / 0.5).clamp(0.0, 1.0);
                        let v = (1.0 - ((local_y - 0.5) / 0.5)).clamp(0.0, 1.0);
                        // Flip u according to face direction for consistent orientation
                        let u = if normal.x > 0.0 { 1.0 - u } else { u };
                        (u, v)
                    } else {
                        // Side of base
                        let u = if normal.x > 0.0 { 1.0 - local_z } else { local_z };
                        let v = (1.0 - (local_y / 0.5)).clamp(0.0, 1.0);
                        (u, v)
                    }
                } else {
                    // Z-facing: front/back faces
                    if local_z >= 0.5 && local_y >= 0.5 {
                        // Front face of upper step
                        let u = local_x.clamp(0.0, 1.0);
                        let v = (1.0 - ((local_y - 0.5) / 0.5)).clamp(0.0, 1.0);
                        let u = if normal.z < 0.0 { 1.0 - u } else { u };
                        (u, v)
                    } else {
                        // Back or base face
                        let u = if normal.z < 0.0 { 1.0 - local_x } else { local_x };
                        let v = (1.0 - local_y).clamp(0.0, 1.0);
                        (u, v)
                    }
                }
            }
            _ => {
                // Standard full-block mapping
                if normal.x.abs() > 0.5 {
                    map_face_x(normal.x)
                } else if normal.y.abs() > 0.5 {
                    map_face_y(normal.y)
                } else {
                    map_face_z(normal.z)
                }
            }
        }
    }
    
    fn raycast(&self, origin: &Point3<f32>, direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>, BlockType, (f32, f32))> {
        let mut origin = *origin;
        let dir = *direction;
        // Si el origen está dentro de un voxel sólido, moverlo ligeramente fuera en la dirección del rayo
        let ox = origin.x.floor() as i32;
        let oy = origin.y.floor() as i32;
        let oz = origin.z.floor() as i32;
        if self.get_block(ox, oy, oz).is_solid() {
            origin = origin + dir * 0.01;
        }
    // Compute a safe max distance: distance from origin to world center + world diagonal
    let center_x = self.width as f32 * 0.5;
    let center_y = self.height as f32 * 0.5;
    let center_z = self.depth as f32 * 0.5;
    let world_center = Vector3::new(center_x, center_y, center_z);
    let world_diag = ((self.width as f32).powi(2) + (self.height as f32).powi(2) + (self.depth as f32).powi(2)).sqrt();
    let max_distance = (world_center - Vector3::new(origin.x, origin.y, origin.z)).magnitude() + world_diag * 1.5;

        let mut x = origin.x.floor() as i32;
        let mut y = origin.y.floor() as i32;
        let mut z = origin.z.floor() as i32;

        let step_x = if dir.x >= 0.0 { 1 } else { -1 };
        let step_y = if dir.y >= 0.0 { 1 } else { -1 };
        let step_z = if dir.z >= 0.0 { 1 } else { -1 };

        let tx_delta = if dir.x.abs() < 1e-6 { f32::INFINITY } else { 1.0 / dir.x.abs() };
        let ty_delta = if dir.y.abs() < 1e-6 { f32::INFINITY } else { 1.0 / dir.y.abs() };
        let tz_delta = if dir.z.abs() < 1e-6 { f32::INFINITY } else { 1.0 / dir.z.abs() };

        let mut tx = if dir.x > 0.0 {
            (x as f32 + 1.0 - origin.x) / dir.x
        } else if dir.x < 0.0 {
            (origin.x - x as f32) / -dir.x
        } else { f32::INFINITY };

        let mut ty = if dir.y > 0.0 {
            (y as f32 + 1.0 - origin.y) / dir.y
        } else if dir.y < 0.0 {
            (origin.y - y as f32) / -dir.y
        } else { f32::INFINITY };

        let mut tz = if dir.z > 0.0 {
            (z as f32 + 1.0 - origin.z) / dir.z
        } else if dir.z < 0.0 {
            (origin.z - z as f32) / -dir.z
        } else { f32::INFINITY };

    // `traveled` was unused; DDA uses `min_t` directly for hit distance.

        loop {
            // Advance to the next voxel boundary: handle ties (corners/edges) by stepping all axes that reach min_t
            let min_t = tx.min(ty).min(tz);
            // safety: if min_t is infinite or beyond max distance, break
            if !min_t.is_finite() || min_t > max_distance { break; }

            // advance axes whose t equals min_t (within epsilon)
            // Make tie epsilon scale slightly with distance to be robust for far-away rays
            let eps = (1e-5_f32).max(min_t * 1e-6);
            let mut stepped_x = false;
            let mut stepped_y = false;
            let mut stepped_z = false;

            if (tx - min_t).abs() <= eps {
                x += step_x;
                tx += tx_delta;
                stepped_x = true;
            }
            if (ty - min_t).abs() <= eps {
                y += step_y;
                ty += ty_delta;
                stepped_y = true;
            }
            if (tz - min_t).abs() <= eps {
                z += step_z;
                tz += tz_delta;
                stepped_z = true;
            }

            // Compute hit point at the boundary
            let hit_point = origin + dir * min_t;
            let block_center = Point3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
            let to_center = Vector3::new(block_center.x - hit_point.x, block_center.y - hit_point.y, block_center.z - hit_point.z);
            let inward_eps = 1e-4_f32 + min_t * 1e-6;

            // Build an estimated face direction (from which axis we stepped)
            let mut face_dir = Vector3::new(0.0, 0.0, 0.0);
            if stepped_x { face_dir.x = -dir.x.signum(); }
            if stepped_y { face_dir.y = -dir.y.signum(); }
            if stepped_z { face_dir.z = -dir.z.signum(); }
            if face_dir.magnitude() > 0.0 { face_dir = face_dir.normalize(); }

            // Candidate sampling strategies (priority): face_dir, -dir, toward center
            let mut hit_point_in = hit_point - dir * inward_eps; // default
            let cand_face = if face_dir.magnitude() > 0.0 { Some(hit_point + face_dir * inward_eps) } else { None };
            let cand_dir = Some(hit_point - dir * inward_eps);
            let cand_center = if to_center.magnitude() > 1e-6 { Some(hit_point + to_center.normalize() * inward_eps) } else { None };

            // Try candidates in order until collision test passes
            if let Some(cf) = cand_face {
                if self.check_special_collision(x, y, z, &cf) { hit_point_in = cf; }
                else if let Some(cd) = cand_dir { if self.check_special_collision(x, y, z, &cd) { hit_point_in = cd; } else if let Some(cc) = cand_center { if self.check_special_collision(x, y, z, &cc) { hit_point_in = cc; } } }
            } else if let Some(cd) = cand_dir {
                if self.check_special_collision(x, y, z, &cd) { hit_point_in = cd; }
                else if let Some(cc) = cand_center { if self.check_special_collision(x, y, z, &cc) { hit_point_in = cc; } }
            } else if let Some(cc) = cand_center { if self.check_special_collision(x, y, z, &cc) { hit_point_in = cc; } }

            // If out of bounds, stop
            if x < 0 || y < 0 || z < 0 || x >= self.width as i32 || y >= self.height as i32 || z >= self.depth as i32 {
                break;
            }

            let block = self.get_block(x, y, z);
            if block.is_solid() && self.check_special_collision(x, y, z, &hit_point_in) {
                // Build normal based on which axis we effectively hit.
                // If multiple axes were stepped (edge/corner), pick the dominant axis
                // (largest absolute component of the ray direction) to get a single-face normal.
                let mut normal = Vector3::new(0.0, 0.0, 0.0);
                let mut stepped_count = 0;
                if stepped_x { normal.x = -dir.x.signum(); stepped_count += 1; }
                if stepped_y { normal.y = -dir.y.signum(); stepped_count += 1; }
                if stepped_z { normal.z = -dir.z.signum(); stepped_count += 1; }

                if stepped_count > 1 {
                    // choose the axis with the largest absolute direction among stepped axes
                    let absx = if stepped_x { dir.x.abs() } else { -1.0 };
                    let absy = if stepped_y { dir.y.abs() } else { -1.0 };
                    let absz = if stepped_z { dir.z.abs() } else { -1.0 };
                    if absx >= absy && absx >= absz {
                        normal.y = 0.0; normal.z = 0.0;
                        normal.x = -dir.x.signum();
                    } else if absy >= absx && absy >= absz {
                        normal.x = 0.0; normal.z = 0.0;
                        normal.y = -dir.y.signum();
                    } else {
                        normal.x = 0.0; normal.y = 0.0;
                        normal.z = -dir.z.signum();
                    }
                }

                if normal.magnitude() > 0.0 { normal = normal.normalize(); }
                let uv = self.calculate_uv(&hit_point_in, &normal, x, y, z);
                return Some((min_t, normal, block, uv));
            }
        }

        None
    }

    // Fallback simple raymarch: march along ray in small steps and test blocks.
    fn raycast_simple(&self, origin: &Point3<f32>, direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>, BlockType, (f32, f32))> {
        // First, quick test: ray vs world AABB. If no intersection, return None (sky)
        let aabb_min = Point3::new(0.0_f32, 0.0_f32, 0.0_f32);
        let aabb_max = Point3::new(self.width as f32, self.height as f32, self.depth as f32);
        if let Some((mut tmin, tmax)) = ray_aabb_intersect(origin, direction, &aabb_min, &aabb_max) {
            // clamp tmin to >= 0
            if tmax < 0.0 { return None; }
            if tmin < 0.0 { tmin = 0.0; }
            let mut t = tmin;
            let step = 0.03_f32; // slightly smaller step to reduce seams while keeping good perf

            while t <= tmax {
                let p = origin + direction * t;
            let x = p.x.floor() as i32;
            let y = p.y.floor() as i32;
            let z = p.z.floor() as i32;

            if x >= 0 && y >= 0 && z >= 0 && x < self.width as i32 && y < self.height as i32 && z < self.depth as i32 {
                let block = self.get_block(x, y, z);
                if block.is_solid() {
                    // Use a slightly inward point for collision tests; bias toward block center to stay inside thin shapes
                    let block_center = Point3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                    let to_center = Vector3::new(block_center.x - p.x, block_center.y - p.y, block_center.z - p.z);
                    let inward_eps = 1e-4_f32 + t * 1e-6;
                    let hit_in = if to_center.magnitude() > 1e-6 {
                        p + to_center.normalize() * inward_eps
                    } else {
                        p - direction * inward_eps
                    };
                    if self.check_special_collision(x, y, z, &hit_in) {
                        // compute a normal using center-based heuristic
                        let mut normal = self.calculate_normal(x, y, z, &hit_in);
                        if normal.magnitude() == 0.0 { normal = Vector3::new(0.0, 1.0, 0.0); }
                        let uv = self.calculate_uv(&hit_in, &normal, x, y, z);
                        return Some((t, normal, block, uv));
                    }
                }
            }

                t += step;
            }
        }

        None
    }

/* ray_aabb_intersect moved below pub fn cast_ray to keep impl VoxelWorld balanced */
    
    #[allow(dead_code)]
    fn calculate_normal(&self, x: i32, y: i32, z: i32, hit_point: &Point3<f32>) -> Vector3<f32> {
        let block_center = Vector3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
        let to_hit = Vector3::new(hit_point.x, hit_point.y, hit_point.z) - block_center;
        
        let abs_x = to_hit.x.abs();
        let abs_y = to_hit.y.abs();
        let abs_z = to_hit.z.abs();
        
        if abs_x > abs_y && abs_x > abs_z {
            Vector3::new(to_hit.x.signum(), 0.0, 0.0)
        } else if abs_y > abs_z {
            Vector3::new(0.0, to_hit.y.signum(), 0.0)
        } else {
            Vector3::new(0.0, 0.0, to_hit.z.signum())
        }
    }
}

#[inline]
pub fn cast_ray(ray_origin: &Point3<f32>, ray_direction: &Vector3<f32>, world: &VoxelWorld, texture_manager: &TextureManager, light_pos: &Point3<f32>, is_day: bool) -> u32 {
    // Prefer the voxel DDA raycast (precise voxel boundaries, avoids seams). Fallback to simple raymarch if needed.
    let hit_opt = world.raycast(ray_origin, ray_direction).or_else(|| world.raycast_simple(ray_origin, ray_direction));

    if let Some((t, normal, block_type, uv)) = hit_opt {
        // Move sampling point slightly inside the voxel along -dir to reduce landing exactly on boundaries
        let eps = 1e-5_f32;
        let hit_point = ray_origin + ray_direction * t;
        let sample_point = hit_point - ray_direction * eps;
        let light_dir = (light_pos - sample_point).normalize();

        // Obtener color de la textura
        let texture = texture_manager.get_texture(block_type);
        let texture_color = texture.sample(uv.0, uv.1);

        // Iluminación
        let diffuse = normal.dot(&light_dir).max(0.0);
        // base ambient depending on day/night
        let base_ambient = if is_day { 0.35 } else { 0.08 };

        // emissive local lighting: scan small neighborhood for emissive blocks and add contribution
        let mut emissive_amount = 0.0_f32;
        let scan_radius = 2i32;
        let hit_block_pos = Point3::new((ray_origin.x + ray_direction.x * t).floor(), (ray_origin.y + ray_direction.y * t).floor(), (ray_origin.z + ray_direction.z * t).floor());
        for oy in -scan_radius..=scan_radius {
            for oz in -scan_radius..=scan_radius {
                for ox in -scan_radius..=scan_radius {
                    let bx = hit_block_pos.x as i32 + ox;
                    let by = hit_block_pos.y as i32 + oy;
                    let bz = hit_block_pos.z as i32 + oz;
                    if bx < 0 || by < 0 || bz < 0 || bx >= world.width as i32 || by >= world.height as i32 || bz >= world.depth as i32 { continue; }
                    let b = world.get_block(bx, by, bz);
                    if b.emits_light() {
                        // contribution falls off with distance
                        let dx = (bx as f32 + 0.5) - (ray_origin.x + ray_direction.x * t);
                        let dy = (by as f32 + 0.5) - (ray_origin.y + ray_direction.y * t);
                        let dz = (bz as f32 + 0.5) - (ray_origin.z + ray_direction.z * t);
                        let dist2 = dx*dx + dy*dy + dz*dz + 1e-4;
                        emissive_amount += 1.0 / dist2;
                    }
                }
            }
        }
        // clamp emissive contribution
        emissive_amount = emissive_amount.min(1.0);

        let ambient = base_ambient + emissive_amount * 0.8 + if block_type.emits_light() { 0.4 } else { 0.0 };
        let intensity = (ambient + diffuse * 0.7).min(1.0);

        let r = ((texture_color[0] as f32 / 255.0 * intensity * 255.0) as u32).min(255);
        let g = ((texture_color[1] as f32 / 255.0 * intensity * 255.0) as u32).min(255);
        let b = ((texture_color[2] as f32 / 255.0 * intensity * 255.0) as u32).min(255);

        0xFF000000 | (r << 16) | (g << 8) | b
    } else {
        if is_day {
            // Day sky gradient (unchanged)
            let gradient = (ray_direction.y * 0.5 + 0.5).max(0.0).min(1.0);
            let r = (135.0 + gradient * 120.0) as u32;
            let g = (206.0 + gradient * 49.0) as u32;
            let b = (235.0 + gradient * 20.0) as u32;
            0xFF000000 | (r << 16) | (g << 8) | b
        } else {
            // Night sky: much darker gradient (horizon slightly lighter), plus sparse stars
            let gradient = (ray_direction.y * 0.5 + 0.5).max(0.0).min(1.0);
            // invert a bit so horizon is slightly less dark
            let night_base = 20.0 + gradient * 40.0; // 20..60
            let night_r = night_base as u32;
            let night_g = (night_base * 0.9) as u32;
            let night_b = (night_base * 1.6).min(255.0) as u32;

            // cheap procedural starfield: hash ray direction to get rare bright pixels
            let s = ray_direction.x * 12.9898 + ray_direction.y * 78.233 + ray_direction.z * 37.719;
            let v = s.sin().abs(); // in [0,1]
            // make stars very sparse
            let star = if v > 0.9995 { 1.0 } else { 0.0 };
            if star > 0.0 {
                // bright star color (slightly bluish)
                let r = ((night_r as f32) + 200.0) as u32; // clamp below when packing
                let g = ((night_g as f32) + 200.0) as u32;
                let b = ((night_b as f32) + 255.0) as u32;
                0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
            } else {
                0xFF000000 | (night_r << 16) | (night_g << 8) | night_b
            }
        }
    }
}

struct Scene {
    world: VoxelWorld,
    texture_manager: TextureManager,
    light_pos: Point3<f32>,
    camera_pos: Point3<f32>,
    camera_target: Point3<f32>,
    // Orbital camera parameters
    orbit_center: Point3<f32>,
    orbit_yaw: f32,
    orbit_radius: f32,
    orbit_height: f32,
    // Stored initial orbit for reset
    initial_orbit_yaw: f32,
    initial_orbit_radius: f32,
    initial_orbit_height: f32,
    is_day: bool,
}

impl Scene {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut world = VoxelWorld::new();
        world.load_from_files()?;
        
        let mut texture_manager = TextureManager::new();
        texture_manager.load_textures()?;
        
        // Calcular el centro de la estructura cargada
        let center_x = world.width as f32 / 2.0;  // 7.5 para 15 bloques
        let center_z = world.depth as f32 / 2.0;  // 9.5 para 19 bloques
        let center_y = world.height as f32 / 2.0; // 6.0 para 12 capas
        
        println!("Centro de la estructura: ({:.1}, {:.1}, {:.1})", center_x, center_y, center_z);
        println!("Dimensiones: {}x{}x{} bloques", world.width, world.height, world.depth);
        
        // Setup an orbital camera around the center
        let orbit_center = Point3::new(center_x, center_y, center_z);
        let orbit_yaw = 0.8_f32; // radians
        let orbit_radius = 13.0_f32;
        let orbit_height = center_y + 6.0_f32;

        // initial camera position/target (computed from orbit)
        let camera_pos = Point3::new(
            orbit_center.x + orbit_radius * orbit_yaw.cos(),
            orbit_height,
            orbit_center.z + orbit_radius * orbit_yaw.sin(),
        );
        let camera_target = orbit_center;

        Ok(Scene {
            // Luz desde arriba y un poco detrás de la cámara
            light_pos: Point3::new(center_x + 10.0, center_y + 12.0, center_z + 15.0),

            camera_pos,
            camera_target,

            orbit_center,
            orbit_yaw,
            orbit_radius,
            orbit_height,
            initial_orbit_yaw: orbit_yaw,
            initial_orbit_radius: orbit_radius,
            initial_orbit_height: orbit_height,
            is_day: true,

            world,
            texture_manager,
        })
    }
    
    fn move_camera(&mut self, delta: Vector3<f32>) {
        self.camera_pos += delta;
    }
    
    fn rotate_camera(&mut self, yaw_delta: f32, pitch_delta: f32) {
        let to_target = self.camera_target - self.camera_pos;
        let distance = to_target.magnitude();
        
        let current_yaw = to_target.z.atan2(to_target.x);
        let current_pitch = (to_target.y / distance).asin();
        
        let new_yaw = current_yaw + yaw_delta;
        let new_pitch = (current_pitch + pitch_delta).max(-1.5).min(1.5);
        
        let new_direction = Vector3::new(
            new_yaw.cos() * new_pitch.cos(),
            new_pitch.sin(),
            new_yaw.sin() * new_pitch.cos()
        ) * distance;
        
        self.camera_target = self.camera_pos + new_direction;
    }

    fn update_camera_from_orbit(&mut self) {
        // Compute camera position from orbital parameters
        self.camera_pos.x = self.orbit_center.x + self.orbit_radius * self.orbit_yaw.cos();
        self.camera_pos.z = self.orbit_center.z + self.orbit_radius * self.orbit_yaw.sin();
        self.camera_pos.y = self.orbit_height;
        self.camera_target = self.orbit_center;
    }
    
    fn render(&self, buffer: &mut Vec<u32>) {
        let fov = std::f32::consts::PI / 3.0;
        let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
        
        let forward = (self.camera_target - self.camera_pos).normalize();
        let right = forward.cross(&Vector3::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(&forward);
        
        let tan_half_fov = (fov * 0.5).tan();

        let x_factors: Vec<f32> = (0..WIDTH).map(|i| {
            (2.0 * (i as f32 + 0.5) / WIDTH as f32 - 1.0) * tan_half_fov * aspect_ratio
        }).collect();
        let y_factors: Vec<f32> = (0..HEIGHT).map(|j| {
            -(2.0 * (j as f32 + 0.5) / HEIGHT as f32 - 1.0) * tan_half_fov
        }).collect();

        buffer.par_chunks_mut(WIDTH).enumerate().for_each(|(j, row)| {
            let y = y_factors[j];
            for i in 0..WIDTH {
                let x = x_factors[i];
                let ray_dir = (forward + right * x + up * y).normalize();
                row[i] = cast_ray(&self.camera_pos, &ray_dir, &self.world, &self.texture_manager, &self.light_pos, self.is_day);
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    
    let mut window = Window::new(
        "Minecraft Raytracer - Con Texturas",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;
    
    window.set_target_fps(60);
    
    let mut scene = Scene::new()?;
    let _move_speed = 1.0;
    let _rotation_speed = 0.05;
    
    println!("==== MINECRAFT RAYTRACER CON TEXTURAS ====");
    println!("Estructura cargada desde carpeta 'capas/'");
    println!("Texturas cargadas desde carpeta 'textures/'");
    
    // Mostrar información de depuración
    println!("\n==== INFORMACIÓN DE DEPURACIÓN ====");
    println!("Posición inicial de cámara: ({:.1}, {:.1}, {:.1})", 
             scene.camera_pos.x, scene.camera_pos.y, scene.camera_pos.z);
    println!("Target de cámara: ({:.1}, {:.1}, {:.1})", 
             scene.camera_target.x, scene.camera_target.y, scene.camera_target.z);
    
    // Verificar si hay bloques cargados
    let mut block_count = 0;
    for y in 0..scene.world.height as i32 {
        for z in 0..scene.world.depth as i32 {
            for x in 0..scene.world.width as i32 {
                if scene.world.get_block(x, y, z).is_solid() {
                    block_count += 1;
                }
            }
        }
    }
    println!("Bloques sólidos encontrados: {}", block_count);
    
    if block_count == 0 {
        println!("⚠️  ADVERTENCIA: No se encontraron bloques sólidos!");
        println!("   Verifica que los archivos en 'capas/' contengan caracteres válidos");
    }
    
    println!("\n==== TEXTURAS ESPERADAS ====");
    println!("En la carpeta 'textures/':");
    println!("• terracota_naranja.png (n)");
    println!("• netherrack.png (i)");
    println!("• bloque_magma.png (l)");
    println!("• terracota_normal.png (t)");
    println!("• terracota_amarilla.png (a)");
    println!("• obsidiana.png (o)");
    println!("• lava.png (p)");
    println!("• bloque_oro.png (y)");
    println!("• cofre.png (c)");
    println!("• piedra.png (e, s) - Compartida para escalera y slab");
    println!("• crying_obsidian.png (k)");
    println!("\n💡 Las escaleras y slabs usan la misma textura 'piedra.png'");
    println!("   pero se mapea automáticamente según la forma del bloque");
    
    println!("\n==== CONTROLES ====");
    println!("A/D - Rotar la cámara alrededor del diorama");
    println!("W/S - Acercar/Alejar la cámara (zoom)");
    println!("Q/E - Subir/bajar la cámara (altura)");
    println!("R - Resetear cámara a órbita inicial");
    println!("N - Toggle día/noche");
    println!("ESC - Salir");
    
    println!("\n==== CONSEJOS DE DEPURACIÓN ====");
    println!("• Si no ves nada, usa Q/E para subir/bajar");
    println!("• Usa IJKL para rotar y buscar la estructura");
    println!("• Presiona R para volver a la posición inicial");
    println!("• El contador de FPS muestra tu posición actual");
    
    let mut frame_count = 0;
    let start_time = std::time::Instant::now();
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Orbital controls
        let yaw_delta = 0.03_f32;
        let radius_delta = 0.3_f32;
        let height_delta = 0.3_f32;

        if window.is_key_down(Key::A) {
            scene.orbit_yaw -= yaw_delta;
        }
        if window.is_key_down(Key::D) {
            scene.orbit_yaw += yaw_delta;
        }
        if window.is_key_down(Key::W) {
            scene.orbit_radius = (scene.orbit_radius - radius_delta).max(2.0);
        }
        if window.is_key_down(Key::S) {
            scene.orbit_radius = (scene.orbit_radius + radius_delta).min(200.0);
        }
        if window.is_key_down(Key::Q) {
            scene.orbit_height += height_delta;
        }
        if window.is_key_down(Key::E) {
            scene.orbit_height -= height_delta;
        }

        // Reset orbital camera
        if window.is_key_down(Key::R) {
            scene.orbit_yaw = scene.initial_orbit_yaw;
            scene.orbit_radius = scene.initial_orbit_radius;
            scene.orbit_height = scene.initial_orbit_height;
            scene.update_camera_from_orbit();
            println!("Cámara orbital reseteada a configuración inicial");
        }

        // Toggle day/night on key press (edge detect)
        static mut PREV_N: bool = false;
        let curr_n = window.is_key_down(Key::N);
        unsafe {
            if curr_n && !PREV_N {
                scene.is_day = !scene.is_day;
                println!("Modo día: {}", scene.is_day);
            }
            PREV_N = curr_n;
        }

        // Update camera position from orbit parameters each frame
        scene.update_camera_from_orbit();

        // (optimizations removed) using robust simple raymarch for all rays
        
        scene.render(&mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;
        
        frame_count += 1;
        if frame_count % 60 == 0 {
            let elapsed = start_time.elapsed().as_secs_f32();
            let fps = frame_count as f32 / elapsed;
            println!("FPS: {:.1} | Pos: ({:.1}, {:.1}, {:.1})", 
                     fps, scene.camera_pos.x, scene.camera_pos.y, scene.camera_pos.z);
        }
    }
    
    Ok(())
}