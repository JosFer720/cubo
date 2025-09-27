use minifb::{Key, Window, WindowOptions};
use nalgebra::{Point3, Vector3};
use image::{RgbImage, Rgb};
use rayon::prelude::*;
use std::fs;

const WIDTH: usize = 600;
const HEIGHT: usize = 450;

/*
Documentación (bloque):
Sección: Tipos y constantes del motor
Descripción: Enumeración de tipos de bloque utilizados en el diorama, colores de fallback y
matriz que indica qué bloques emiten luz. Estos valores sirven como referencia para cargar
texturas, calcular materiales y determinar fuentes emisivas en el sombreado.
*/

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum BlockType {
    Aire = 0,
    TerracotaNaranja = 1,
    Netherrack = 2,
    BloqueMagma = 3,
    TerracotaNormal = 4,
    TerracotaAmarilla = 5,
    ObsidianaNormal = 6,
    Lava = 7,
    BloqueOro = 8,
    Cofre = 9,
    EscaleraPiedra = 10,
    SlabPiedra = 11,
    CryingObsidian = 12,
}


static BLOCK_COLORS: [(f32, f32, f32); 13] = [
    (0.0, 0.0, 0.0),
    (0.8, 0.4, 0.1),
    (0.4, 0.2, 0.2),
    (0.9, 0.3, 0.1),
    (0.6, 0.4, 0.3),
    (0.9, 0.8, 0.2),
    (0.1, 0.1, 0.2),
    (1.0, 0.5, 0.0),
    (1.0, 0.8, 0.0),
    (0.5, 0.3, 0.2),
    (0.5, 0.5, 0.5),
    (0.6, 0.6, 0.6),
    (0.3, 0.1, 0.4),
];

static EMISSIVE_BLOCKS: [bool; 13] = [
    false, false, false, true,
    false, false, false, true,
    false, false, false, false, false,
];

/*
Documentación (bloque):
Sección: Gestor de texturas
Descripción: Implementa carga de imágenes desde la carpeta `textures/`. Si falta algún
archivo, genera un marcador (placeholder) básico en disco y utiliza un color de fallback.
Proporciona métodos para obtener una textura por `BlockType` y muestrear colores UV.
*/

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
        /*
        Descripción: Lista de nombres esperados para las texturas. Si faltan, se crea un
        PNG de marcador con color de fallback. Los índices coinciden con `BlockType`.
        */

        let texture_files = [
            "aire.png",
            "terracota_naranja.png",
            "netherrack.png",
            "bloque_magma.png",
            "terracota_normal.png",
            "terracota_amarilla.png",
            "obsidiana.png",
            "lava.png",
            "bloque_oro.png",
            "cofre.png",
            "piedra.png",
            "piedra.png",
            "crying_obsidian.png",
        ];


        let _ = std::fs::create_dir_all("textures");

        for (i, filename) in texture_files.iter().enumerate() {
            let path = format!("textures/{}", filename);

                /*
                Si el archivo no existe, se genera un PNG de marcador de 16×16 con el color
                base definido en `BLOCK_COLORS`. Esto ayuda a evitar fallos visuales cuando
                faltan recursos.
                */
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

                if let Err(e) = img.save(&path) {
                    println!("⚠ No se pudo crear marcador {}: {}", path, e);
                } else {
                    println!("ℹ Marcador creado: {}", path);
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


#[derive(Debug, Clone, Copy)]
pub struct MaterialProperties {
    pub albedo: Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub reflectance: f32,
    pub emissive_strength: f32,
    pub specular_color: Vector3<f32>,
}

impl MaterialProperties {
    pub fn new(albedo: Vector3<f32>, metallic: f32, roughness: f32, reflectance: f32, emissive_strength: f32, specular_color: Vector3<f32>) -> Self {
        Self { albedo, metallic, roughness, reflectance, emissive_strength, specular_color }
    }
}


pub fn get_material(block: BlockType) -> MaterialProperties {
    match block {
        BlockType::BloqueOro => {

            MaterialProperties::new(
                Vector3::new(1.0, 0.766, 0.336),
                0.9,
                0.12,
                0.8,
                0.0,
                Vector3::new(1.0, 0.85, 0.45),
            )
        }
        BlockType::ObsidianaNormal => {

            MaterialProperties::new(
                Vector3::new(0.15, 0.12, 0.12),
                0.0,
                0.03,
                0.95,
                0.0,
                Vector3::new(0.9, 0.9, 0.95),
            )
        }
        BlockType::CryingObsidian => {

            MaterialProperties::new(
                Vector3::new(0.30, 0.05, 0.45),
                0.0,
                0.12,
                0.55,
                0.08,
                Vector3::new(0.6, 0.2, 0.8),
            )
        }
        BlockType::Lava => {

            MaterialProperties::new(
                Vector3::new(0.9, 0.4, 0.1),
                0.0,
                0.35,
                0.25,
                1.5,
                Vector3::new(1.0, 0.6, 0.2),
            )
        }
        BlockType::BloqueMagma => {

            MaterialProperties::new(
                Vector3::new(0.5, 0.2, 0.1),
                0.0,
                0.28,
                0.28,
                0.8,
                Vector3::new(0.7, 0.3, 0.15),
            )
        }
        _ => {

            MaterialProperties::new(
                Vector3::new(0.8, 0.8, 0.8),
                0.0,
                0.9,
                0.04,
                0.0,
                Vector3::new(0.04, 0.04, 0.04),
            )
        }
    }
}


#[inline]
fn reflect(dir: &Vector3<f32>, normal: &Vector3<f32>) -> Vector3<f32> {
    dir - normal * (2.0 * dir.dot(normal))
}



#[inline]
fn fresnel_simple(view_dir: &Vector3<f32>, normal: &Vector3<f32>, material: &MaterialProperties) -> f32 {
    let v_dot_n = view_dir.dot(normal).max(0.0);

    let f0_vec = Vector3::new(0.04, 0.04, 0.04).lerp(&material.specular_color, material.metallic);

    let f0 = (f0_vec.x + f0_vec.y + f0_vec.z) / 3.0;

    f0 + (1.0 - f0) * (1.0 - v_dot_n).powf(5.0)
}


fn ray_aabb_intersect(orig: &Point3<f32>, dir: &Vector3<f32>, min: &Point3<f32>, max: &Point3<f32>) -> Option<(f32,f32)> {

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

    /*
    Documentación (bloque):
    Sección: Carga del mundo desde archivos
    Descripción: Lee los ficheros de la carpeta `capas/` y construye la malla voxel interna
    (`blocks`) con dimensiones fijas esperadas. Si falta una capa, se sustituye por una
    capa vacía para mantener la geometría.
    */
    fn load_from_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut layers = Vec::new();


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
        let mut local_x = point.x - x as f32;
        let mut local_y = point.y - y as f32;
        let mut local_z = point.z - z as f32;

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

    /*
    Documentación (bloque):
    Sección: Mapeo UV
    Descripción: Calcula las coordenadas UV para el punto de impacto según la cara hitada y
    maneja casos especiales para slabs y escaleras, devolviendo un par (u,v) en [0,1].
    */

    fn calculate_uv(&self, hit_point: &Point3<f32>, normal: &Vector3<f32>, block_x: i32, block_y: i32, block_z: i32) -> (f32, f32) {
        let block_type = self.get_block(block_x, block_y, block_z);
        let local_x = hit_point.x - block_x as f32;
        let local_y = hit_point.y - block_y as f32;
        let local_z = hit_point.z - block_z as f32;

        let map_face_x = |nx: f32| {

            let u = if nx > 0.0 { 1.0 - local_z } else { local_z };
            let v = (1.0 - local_y).clamp(0.0, 1.0);
            (u.clamp(0.0, 1.0), v)
        };

        let map_face_y = |ny: f32| {

            let u = local_x.clamp(0.0, 1.0);
            let v = if ny > 0.0 { 1.0 - local_z } else { local_z };
            (u, v.clamp(0.0, 1.0))
        };

        let map_face_z = |nz: f32| {

            let u = if nz < 0.0 { 1.0 - local_x } else { local_x };
            let v = (1.0 - local_y).clamp(0.0, 1.0);
            (u.clamp(0.0, 1.0), v)
        };

        match block_type {
            BlockType::SlabPiedra => {

                if normal.y.abs() > 0.5 {

                    map_face_y(normal.y)
                } else if normal.x.abs() > 0.5 {

                    let u = if normal.x > 0.0 { 1.0 - local_z } else { local_z };
                    let v = (1.0 - (local_y / 0.5)).clamp(0.0, 1.0);
                    (u.clamp(0.0, 1.0), v)
                } else {

                    let u = if normal.z < 0.0 { 1.0 - local_x } else { local_x };
                    let v = (1.0 - (local_y / 0.5)).clamp(0.0, 1.0);
                    (u.clamp(0.0, 1.0), v)
                }
            }
            BlockType::EscaleraPiedra => {

                if normal.y > 0.5 {

                    if local_z >= 0.5 {

                        let u = local_x.clamp(0.0, 1.0);
                        let v = ((local_z - 0.5) / 0.5).clamp(0.0, 1.0);
                        (u, v)
                    } else {

                        let u = local_x.clamp(0.0, 1.0);
                        let v = (local_z / 0.5).clamp(0.0, 1.0);
                        (u, v)
                    }
                } else if normal.x.abs() > 0.5 {

                    if local_z >= 0.5 && local_y >= 0.5 {
                        let u = ((local_z - 0.5) / 0.5).clamp(0.0, 1.0);
                        let v = (1.0 - ((local_y - 0.5) / 0.5)).clamp(0.0, 1.0);

                        let u = if normal.x > 0.0 { 1.0 - u } else { u };
                        (u, v)
                    } else {

                        let u = if normal.x > 0.0 { 1.0 - local_z } else { local_z };
                        let v = (1.0 - (local_y / 0.5)).clamp(0.0, 1.0);
                        (u, v)
                    }
                } else {

                    if local_z >= 0.5 && local_y >= 0.5 {

                        let u = local_x.clamp(0.0, 1.0);
                        let v = (1.0 - ((local_y - 0.5) / 0.5)).clamp(0.0, 1.0);
                        let u = if normal.z < 0.0 { 1.0 - u } else { u };
                        (u, v)
                    } else {

                        let u = if normal.z < 0.0 { 1.0 - local_x } else { local_x };
                        let v = (1.0 - local_y).clamp(0.0, 1.0);
                        (u, v)
                    }
                }
            }
            _ => {

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

    /*
    Documentación (bloque):
    Sección: Raycasting DDA (principal)
    Descripción: Implementa un DDA robusto para recorrer voxels. Maneja ties (esquinas),
    intenta varios puntos candidatos para evitar fallas con geometrías delgadas y devuelve
    distancia t, normal de la cara, tipo de bloque y coordenadas UV si hay impacto.
    */
    fn raycast(&self, origin: &Point3<f32>, direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>, BlockType, (f32, f32))> {
        let mut origin = *origin;
        let dir = *direction;

        let ox = origin.x.floor() as i32;
        let oy = origin.y.floor() as i32;
        let oz = origin.z.floor() as i32;
        if self.get_block(ox, oy, oz).is_solid() {
            origin = origin + dir * 0.01;
        }

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



        loop {

            let min_t = tx.min(ty).min(tz);

            if !min_t.is_finite() || min_t > max_distance { break; }



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


            let hit_point = origin + dir * min_t;
            let block_center = Point3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
            let to_center = Vector3::new(block_center.x - hit_point.x, block_center.y - hit_point.y, block_center.z - hit_point.z);
            let inward_eps = 1e-4_f32 + min_t * 1e-6;


            let mut face_dir = Vector3::new(0.0, 0.0, 0.0);
            if stepped_x { face_dir.x = -dir.x.signum(); }
            if stepped_y { face_dir.y = -dir.y.signum(); }
            if stepped_z { face_dir.z = -dir.z.signum(); }
            if face_dir.magnitude() > 0.0 { face_dir = face_dir.normalize(); }


            let mut hit_point_in = hit_point - dir * inward_eps;
            let cand_face = if face_dir.magnitude() > 0.0 { Some(hit_point + face_dir * inward_eps) } else { None };
            let cand_dir = Some(hit_point - dir * inward_eps);
            let cand_center = if to_center.magnitude() > 1e-6 { Some(hit_point + to_center.normalize() * inward_eps) } else { None };


            if let Some(cf) = cand_face {
                if self.check_special_collision(x, y, z, &cf) { hit_point_in = cf; }
                else if let Some(cd) = cand_dir { if self.check_special_collision(x, y, z, &cd) { hit_point_in = cd; } else if let Some(cc) = cand_center { if self.check_special_collision(x, y, z, &cc) { hit_point_in = cc; } } }
            } else if let Some(cd) = cand_dir {
                if self.check_special_collision(x, y, z, &cd) { hit_point_in = cd; }
                else if let Some(cc) = cand_center { if self.check_special_collision(x, y, z, &cc) { hit_point_in = cc; } }
            } else if let Some(cc) = cand_center { if self.check_special_collision(x, y, z, &cc) { hit_point_in = cc; } }


            if x < 0 || y < 0 || z < 0 || x >= self.width as i32 || y >= self.height as i32 || z >= self.depth as i32 {
                break;
            }

            let block = self.get_block(x, y, z);
            if block.is_solid() && self.check_special_collision(x, y, z, &hit_point_in) {



                let mut normal = Vector3::new(0.0, 0.0, 0.0);
                let mut stepped_count = 0;
                if stepped_x { normal.x = -dir.x.signum(); stepped_count += 1; }
                if stepped_y { normal.y = -dir.y.signum(); stepped_count += 1; }
                if stepped_z { normal.z = -dir.z.signum(); stepped_count += 1; }

                if stepped_count > 1 {

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

    /*
    Documentación (bloque):
    Sección: Raymarch simple (fallback)
    Descripción: Método alternativo que avanza en pasos constantes a lo largo del rayo. Es
    menos eficiente pero más simple; se usa cuando DDA no encuentra impacto por razones
    numéricas o para cubrir casos extremos.
    */
    fn raycast_simple(&self, origin: &Point3<f32>, direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>, BlockType, (f32, f32))> {

        let aabb_min = Point3::new(0.0_f32, 0.0_f32, 0.0_f32);
        let aabb_max = Point3::new(self.width as f32, self.height as f32, self.depth as f32);
        if let Some((mut tmin, tmax)) = ray_aabb_intersect(origin, direction, &aabb_min, &aabb_max) {

            if tmax < 0.0 { return None; }
            if tmin < 0.0 { tmin = 0.0; }
            let mut t = tmin;
            let step = 0.03_f32;

            while t <= tmax {
                let p = origin + direction * t;
            let x = p.x.floor() as i32;
            let y = p.y.floor() as i32;
            let z = p.z.floor() as i32;

            if x >= 0 && y >= 0 && z >= 0 && x < self.width as i32 && y < self.height as i32 && z < self.depth as i32 {
                let block = self.get_block(x, y, z);
                if block.is_solid() {

                    let block_center = Point3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                    let to_center = Vector3::new(block_center.x - p.x, block_center.y - p.y, block_center.z - p.z);
                    let inward_eps = 1e-4_f32 + t * 1e-6;
                    let hit_in = if to_center.magnitude() > 1e-6 {
                        p + to_center.normalize() * inward_eps
                    } else {
                        p - direction * inward_eps
                    };
                    if self.check_special_collision(x, y, z, &hit_in) {

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

/*
Nota: `ray_aabb_intersect` se define fuera de esta implementación para mantener la
organización del archivo. Ver función más arriba.
*/

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
/*
Documentación (bloque):
Sección: Trazado de rayo y sombreado
Descripción: `cast_ray` expone la entrada del renderer. Internamente define `shade`, una
función recursiva que aplica PBR simplificado, fresnel, reflexiones y añade contribución
emisiva a partir de un escaneo local. Si no hay impacto, solicita color al `Skybox`.
*/
pub fn cast_ray(ray_origin: &Point3<f32>, ray_direction: &Vector3<f32>, world: &VoxelWorld, texture_manager: &TextureManager, light_pos: &Point3<f32>, is_day: bool) -> u32 {

    fn pack_color(col: Vector3<f32>) -> u32 {
        let r = (col.x.max(0.0).min(1.0) * 255.0) as u32;
        let g = (col.y.max(0.0).min(1.0) * 255.0) as u32;
        let b = (col.z.max(0.0).min(1.0) * 255.0) as u32;
        0xFF000000 | (r << 16) | (g << 8) | b
    }

    let max_bounces = 3usize;


    fn shade(origin: Point3<f32>, dir: Vector3<f32>, depth: usize, max_bounces: usize, world: &VoxelWorld, texture_manager: &TextureManager, light_pos: &Point3<f32>, is_day: bool) -> Vector3<f32> {

        if let Some((t, normal, block_type, uv)) = world.raycast(&origin, &dir).or_else(|| world.raycast_simple(&origin, &dir)) {
            let eps = 1e-5_f32;
            let hit_point = origin + dir * t;
            let sample_point = hit_point - dir * eps;


            let texture = texture_manager.get_texture(block_type);
            let tex_rgb = texture.sample(uv.0, uv.1);
            let tex_col = Vector3::new(tex_rgb[0] as f32 / 255.0, tex_rgb[1] as f32 / 255.0, tex_rgb[2] as f32 / 255.0);
            let material = get_material(block_type);
            let base_color = tex_col.component_mul(&material.albedo);


            let light_dir = (light_pos - sample_point).normalize();
            let view_dir = -dir.normalize();
            let half = (light_dir + view_dir).normalize();

            let n_dot_l = normal.dot(&light_dir).max(0.0);
            let n_dot_h = normal.dot(&half).max(0.0);


            let shininess = (1.0 - material.roughness).max(0.01) * 256.0;
            let spec_term = n_dot_h.powf(shininess);


            let fresnel = fresnel_simple(&view_dir, &normal, &material);


            let diffuse = base_color * (1.0 - material.metallic) * n_dot_l;


            let specular = material.specular_color * (spec_term * material.reflectance * fresnel);


            let mut emissive_amount = 0.0_f32;

            let scan_radius = 4i32;
            let hit_block_pos = Point3::new(hit_point.x.floor(), hit_point.y.floor(), hit_point.z.floor());
            for oy in -scan_radius..=scan_radius {
                for oz in -scan_radius..=scan_radius {
                    for ox in -scan_radius..=scan_radius {
                        let bx = hit_block_pos.x as i32 + ox;
                        let by = hit_block_pos.y as i32 + oy;
                        let bz = hit_block_pos.z as i32 + oz;
                        if bx < 0 || by < 0 || bz < 0 || bx >= world.width as i32 || by >= world.height as i32 || bz >= world.depth as i32 { continue; }
                        let b = world.get_block(bx, by, bz);
                        if b.emits_light() {
                            let dx = (bx as f32 + 0.5) - hit_point.x;
                            let dy = (by as f32 + 0.5) - hit_point.y;
                            let dz = (bz as f32 + 0.5) - hit_point.z;
                            let dist2 = dx*dx + dy*dy + dz*dz + 1e-4;

                            emissive_amount += 1.5 / dist2;
                        }
                    }
                }
            }
            emissive_amount = emissive_amount.min(1.0);


            let base_ambient = if is_day { 0.35 } else { 0.08 };

            let ambient = base_ambient + emissive_amount * 1.0 + if block_type.emits_light() { 0.25 } else { 0.0 };


            let mut color = Vector3::new(0.0,0.0,0.0);
            color += diffuse * 0.9;
            color += specular * 1.0;


            if material.reflectance > 0.1 && depth < max_bounces {
                let reflect_dir = reflect(&dir, &normal).normalize();
                let reflect_origin = hit_point + normal * 0.001;
                let reflected = shade(reflect_origin, reflect_dir, depth + 1, max_bounces, world, texture_manager, light_pos, is_day);

                let refl_boost = if material.roughness < 0.1 { 1.2 } else { 1.0 };

                color += reflected * material.reflectance * fresnel * refl_boost;
            }


            color += base_color.component_mul(&Vector3::new(ambient,ambient,ambient)) * 0.6;
            color += material.albedo * material.emissive_strength;


            if !is_day && material.emissive_strength > 0.5 {
                let night_boost = match block_type {
                    BlockType::Lava => 0.2,
                    BlockType::BloqueMagma => 0.15,
                    _ => 0.1,
                };
                color += material.albedo * material.emissive_strength * night_boost;
            }


            return Vector3::new(color.x.min(1.0).max(0.0), color.y.min(1.0).max(0.0), color.z.min(1.0).max(0.0));
        }


        let sky = Skybox::new(*light_pos - Point3::new(0.0, 0.0, 0.0));
        let sky_col = sky.sample(&dir, is_day);
        return sky_col;
    }

    let col = shade(*ray_origin, *ray_direction, 0usize, max_bounces, world, texture_manager, light_pos, is_day);
    pack_color(col)
}


pub struct Skybox {
    pub day_horizon: Vector3<f32>,
    pub day_zenith: Vector3<f32>,
    pub night_horizon: Vector3<f32>,
    pub night_zenith: Vector3<f32>,
    pub sun_direction: Vector3<f32>,
    pub sun_intensity: f32,
}

impl Skybox {
    pub fn new(sun_dir: Vector3<f32>) -> Self {
        Skybox {
            day_horizon: Vector3::new(1.0, 0.6, 0.3),
            day_zenith: Vector3::new(0.3, 0.7, 1.0),
            night_horizon: Vector3::new(0.2, 0.1, 0.3),
            night_zenith: Vector3::new(0.01, 0.01, 0.05),
            sun_direction: sun_dir.normalize(),
            sun_intensity: 0.5,
        }
    }


    pub fn sample(&self, direction: &Vector3<f32>, is_day: bool) -> Vector3<f32> {

        let t = (direction.y * 0.5 + 0.5).max(0.0).min(1.0);


        let horizon = if is_day { self.day_horizon } else { self.night_horizon };
        let zenith = if is_day { self.day_zenith } else { self.night_zenith };


        let t_smooth = t * t * (3.0 - 2.0 * t);
        let mut col = horizon * (1.0 - t_smooth) + zenith * t_smooth;


        let sun_dot = direction.normalize().dot(&self.sun_direction).max(0.0);
        let sun_contrib = sun_dot.powf(16.0) * self.sun_intensity * 0.5;
        if sun_contrib > 0.0 {

            let sun_color = if is_day { Vector3::new(1.0, 0.95, 0.85) } else { Vector3::new(0.8, 0.7, 0.6) * 0.5 };
            col += sun_color * sun_contrib;
        }


        if !is_day {

            let s = direction.x * 12.9898 + direction.y * 78.233 + direction.z * 37.719;
            let star_hash = s.sin().abs();
            if star_hash > 0.9995 {

                let brightness = ((star_hash - 0.9995) / (1.0 - 0.9995)).clamp(0.0, 1.0);

                let star_col = Vector3::new(1.0, 1.0, 1.05);
                col += star_col * (0.7 + 0.3 * brightness);
            }
        }


        Vector3::new(col.x.max(0.0).min(1.0), col.y.max(0.0).min(1.0), col.z.max(0.0).min(1.0))
    }
}

struct Scene {
    world: VoxelWorld,
    texture_manager: TextureManager,
    light_pos: Point3<f32>,
    camera_pos: Point3<f32>,
    camera_target: Point3<f32>,

    orbit_center: Point3<f32>,
    orbit_yaw: f32,
    orbit_radius: f32,
    orbit_height: f32,

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


        let center_x = world.width as f32 / 2.0;
        let center_z = world.depth as f32 / 2.0;
        let center_y = world.height as f32 / 2.0;

        println!("Centro de la estructura: ({:.1}, {:.1}, {:.1})", center_x, center_y, center_z);
        println!("Dimensiones: {}x{}x{} bloques", world.width, world.height, world.depth);


        let orbit_center = Point3::new(center_x, center_y, center_z);
        let orbit_yaw = 0.8_f32;
        let orbit_radius = 13.0_f32;
        let orbit_height = center_y + 6.0_f32;


        let camera_pos = Point3::new(
            orbit_center.x + orbit_radius * orbit_yaw.cos(),
            orbit_height,
            orbit_center.z + orbit_radius * orbit_yaw.sin(),
        );
        let camera_target = orbit_center;

        Ok(Scene {

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


    println!("\n==== INFORMACIÓN DE DEPURACIÓN ====");
    println!("Posición inicial de cámara: ({:.1}, {:.1}, {:.1})",
             scene.camera_pos.x, scene.camera_pos.y, scene.camera_pos.z);
    println!("Target de cámara: ({:.1}, {:.1}, {:.1})",
             scene.camera_target.x, scene.camera_target.y, scene.camera_target.z);


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


        if window.is_key_down(Key::R) {
            scene.orbit_yaw = scene.initial_orbit_yaw;
            scene.orbit_radius = scene.initial_orbit_radius;
            scene.orbit_height = scene.initial_orbit_height;
            scene.update_camera_from_orbit();
            println!("Cámara orbital reseteada a configuración inicial");
        }


        static mut PREV_N: bool = false;
        let curr_n = window.is_key_down(Key::N);
        unsafe {
            if curr_n && !PREV_N {
                scene.is_day = !scene.is_day;
                println!("Modo día: {}", scene.is_day);
            }
            PREV_N = curr_n;
        }


        scene.update_camera_from_orbit();



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
