use minifb::{Key, Window, WindowOptions};
use nalgebra::{Point3, Vector3};
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

// Arrays estáticos para lookup rápido
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
            ' ' => BlockType::Aire,  // Espacio en blanco = aire
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
                    // Crear capa vacía de aire
                    layers.push("               \n".repeat(19));
                }
            }
        }
        
        self.load_from_layer_data(&layers);
        Ok(())
    }
    
    fn load_from_layer_data(&mut self, layer_data: &[String]) {
        if layer_data.is_empty() {
            return;
        }
        
        // Establecer dimensiones basadas en tus archivos
        self.width = 15;  // 15 caracteres por línea
        self.depth = 19;  // 19 líneas por archivo
        self.height = layer_data.len(); // 12 capas
        
        self.blocks = vec![BlockType::Aire; self.width * self.height * self.depth];
        
        // Cargar cada capa (y=0 es la base, y=11 es la cima)
        for (y, layer_str) in layer_data.iter().enumerate() {
            let lines: Vec<&str> = layer_str.lines().collect();
            for (z, line) in lines.iter().enumerate() {
                if z < self.depth {
                    // Asegurar que la línea tenga exactamente 15 caracteres
                    let padded_line = format!("{:15}", line);
                    for (x, ch) in padded_line.chars().take(self.width).enumerate() {
                        let block_type = BlockType::from_char(ch);
                        let index = y * (self.width * self.depth) + z * self.width + x;
                        self.blocks[index] = block_type;
                    }
                }
            }
        }
        
        println!("Mundo cargado: {}x{}x{}", self.width, self.height, self.depth);
        println!("Total de bloques: {}", self.blocks.len());
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
    
    // Función para verificar colisión con formas especiales
    fn check_special_collision(&self, x: i32, y: i32, z: i32, point: &Point3<f32>) -> bool {
        let block = self.get_block(x, y, z);
        
        // Coordenadas locales dentro del bloque (0.0 a 1.0)
        let local_x = point.x - x as f32;
        let local_y = point.y - y as f32;
        let local_z = point.z - z as f32;
        
        match block {
            BlockType::SlabPiedra => {
                // Slab ocupa solo la mitad inferior del bloque
                local_y <= 0.5
            },
            BlockType::EscaleraPiedra => {
                // Escalera simple: mitad inferior + escalón en una esquina
                if local_y <= 0.5 {
                    true  // Base de la escalera
                } else if local_y <= 1.0 && local_z >= 0.5 {
                    true  // Escalón superior
                } else {
                    false
                }
            },
            _ => {
                // Bloque completo
                local_x >= 0.0 && local_x <= 1.0 && 
                local_y >= 0.0 && local_y <= 1.0 && 
                local_z >= 0.0 && local_z <= 1.0
            }
        }
    }
    
    // DDA Algorithm optimizado para voxel traversal con formas especiales
    fn raycast(&self, origin: &Point3<f32>, direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>, BlockType)> {
        // Fast 3D DDA voxel traversal
        let dir = *direction;
        let max_distance = 100.0;

        // Starting voxel
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

        let mut traveled = 0.0_f32;

        loop {
            if traveled > max_distance { break; }

            // Check voxel
            let block = self.get_block(x, y, z);
            let hit_point = origin + dir * traveled.max(0.0);
            if block.is_solid() && self.check_special_collision(x, y, z, &hit_point) {
                let normal = self.calculate_normal(x, y, z, &hit_point);
                return Some((traveled.max(0.0), normal, block));
            }

            // Step to next voxel boundary
            if tx < ty {
                if tx < tz {
                    x += step_x;
                    traveled = tx;
                    tx += tx_delta;
                } else {
                    z += step_z;
                    traveled = tz;
                    tz += tz_delta;
                }
            } else {
                if ty < tz {
                    y += step_y;
                    traveled = ty;
                    ty += ty_delta;
                } else {
                    z += step_z;
                    traveled = tz;
                    tz += tz_delta;
                }
            }

            // Early out if we leave reasonable bounds
            if x < -2 || y < -2 || z < -2 || x > self.width as i32 + 2 || y > self.height as i32 + 2 || z > self.depth as i32 + 2 {
                break;
            }
        }

        None
    }
    
    fn calculate_normal(&self, x: i32, y: i32, z: i32, hit_point: &Point3<f32>) -> Vector3<f32> {
        let block_center = Vector3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
        let to_hit = Vector3::new(hit_point.x, hit_point.y, hit_point.z) - block_center;
        
        // Determinar qué cara fue golpeada
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
pub fn cast_ray(ray_origin: &Point3<f32>, ray_direction: &Vector3<f32>, world: &VoxelWorld, light_pos: &Point3<f32>) -> u32 {
    if let Some((t, normal, block_type)) = world.raycast(ray_origin, ray_direction) {
        let hit_point = ray_origin + ray_direction * t;
        let light_dir = (light_pos - hit_point).normalize();
        
        // Iluminación mejorada
        let diffuse = normal.dot(&light_dir).max(0.0);
        let ambient = if block_type.emits_light() { 0.9 } else { 0.3 };
        let intensity = (ambient + diffuse * 0.7).min(1.0);
        
        let base_color = block_type.get_color();
        let r = ((base_color.0 * intensity * 255.0) as u32).min(255);
        let g = ((base_color.1 * intensity * 255.0) as u32).min(255);
        let b = ((base_color.2 * intensity * 255.0) as u32).min(255);
        
        0xFF000000 | (r << 16) | (g << 8) | b
    } else {
        // Cielo degradado
        let gradient = (ray_direction.y * 0.5 + 0.5).max(0.0).min(1.0);
        let r = (135.0 + gradient * 120.0) as u32;  // Azul cielo
        let g = (206.0 + gradient * 49.0) as u32;
        let b = (235.0 + gradient * 20.0) as u32;
        0xFF000000 | (r << 16) | (g << 8) | b
    }
}

struct Scene {
    world: VoxelWorld,
    light_pos: Point3<f32>,
    camera_pos: Point3<f32>,
    camera_target: Point3<f32>,
}

impl Scene {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut world = VoxelWorld::new();
        world.load_from_files()?;
        
        Ok(Scene {
            light_pos: Point3::new(7.5, 25.0, 7.5),
            camera_pos: Point3::new(30.0, 20.0, 30.0),
            camera_target: Point3::new(7.5, 8.0, 9.5),
            world,
        })
    }
    
    fn move_camera(&mut self, delta: Vector3<f32>) {
        self.camera_pos += delta;
    }
    
    fn rotate_camera(&mut self, yaw_delta: f32, pitch_delta: f32) {
        let to_target = self.camera_target - self.camera_pos;
        let distance = to_target.magnitude();
        
        // Calcular ángulos actuales
        let current_yaw = to_target.z.atan2(to_target.x);
        let current_pitch = (to_target.y / distance).asin();
        
        // Aplicar deltas
        let new_yaw = current_yaw + yaw_delta;
        let new_pitch = (current_pitch + pitch_delta).max(-1.5).min(1.5);
        
        // Calcular nueva posición del target
        let new_direction = Vector3::new(
            new_yaw.cos() * new_pitch.cos(),
            new_pitch.sin(),
            new_yaw.sin() * new_pitch.cos()
        ) * distance;
        
        self.camera_target = self.camera_pos + new_direction;
    }
    
    fn render(&self, buffer: &mut Vec<u32>) {
        let fov = std::f32::consts::PI / 3.0;
        let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
        
        let forward = (self.camera_target - self.camera_pos).normalize();
        let right = forward.cross(&Vector3::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(&forward);
        
        let tan_half_fov = (fov * 0.5).tan();

        // Precompute X factors per column and Y factors per row to avoid recomputing division
        let x_factors: Vec<f32> = (0..WIDTH).map(|i| {
            (2.0 * (i as f32 + 0.5) / WIDTH as f32 - 1.0) * tan_half_fov * aspect_ratio
        }).collect();
        let y_factors: Vec<f32> = (0..HEIGHT).map(|j| {
            -(2.0 * (j as f32 + 0.5) / HEIGHT as f32 - 1.0) * tan_half_fov
        }).collect();

        // Normalize camera basis once
        let fwd = forward; // already normalized
        let rgt = right; // already normalized
        let upv = up; // orthogonal

        // Parallelize over rows
        buffer.par_chunks_mut(WIDTH).enumerate().for_each(|(j, row)| {
            let y = y_factors[j];
            for i in 0..WIDTH {
                let x = x_factors[i];
                let ray_dir = (fwd + rgt * x + upv * y).normalize();
                row[i] = cast_ray(&self.camera_pos, &ray_dir, &self.world, &self.light_pos);
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    
    let mut window = Window::new(
        "Minecraft Raytracer - Tu Estructura",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;
    
    window.set_target_fps(60);
    
    let mut scene = Scene::new()?;
    let move_speed = 1.0;
    let rotation_speed = 0.05;
    
    println!("==== MINECRAFT RAYTRACER ====");
    println!("Estructura cargada desde carpeta 'capas/'");
    println!("Dimensiones: 15x12x19 bloques");
    
    println!("\n==== CONTROLES ====");
    println!("WASD - Mover cámara (adelante/atrás/izquierda/derecha)");
    println!("Q/E - Subir/bajar cámara");
    println!("IJKL - Rotar cámara (I=arriba, K=abajo, J=izquierda, L=derecha)");
    println!("ESC - Salir");
    
    println!("\n==== BLOQUES ESPECIALES ====");
    println!("s = Slab (medio bloque)");
    println!("e = Escalera");
    println!("l = Magma (brilla)");
    println!("p = Lava (brilla)");
    println!("Espacio = Aire");
    
    let mut frame_count = 0;
    let start_time = std::time::Instant::now();
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Controles de movimiento simplificados
        if window.is_key_down(Key::A) {
            scene.move_camera(Vector3::new(-move_speed, 0.0, 0.0));
        }
        if window.is_key_down(Key::D) {
            scene.move_camera(Vector3::new(move_speed, 0.0, 0.0));
        }
        if window.is_key_down(Key::W) {
            scene.move_camera(Vector3::new(0.0, 0.0, -move_speed));
        }
        if window.is_key_down(Key::S) {
            scene.move_camera(Vector3::new(0.0, 0.0, move_speed));
        }
        if window.is_key_down(Key::Q) {
            scene.move_camera(Vector3::new(0.0, move_speed, 0.0));
        }
        if window.is_key_down(Key::E) {
            scene.move_camera(Vector3::new(0.0, -move_speed, 0.0));
        }
        
        // Controles de rotación con IJKL
        if window.is_key_down(Key::J) {
            scene.rotate_camera(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::L) {
            scene.rotate_camera(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::I) {
            scene.rotate_camera(0.0, rotation_speed);
        }
        if window.is_key_down(Key::K) {
            scene.rotate_camera(0.0, -rotation_speed);
        }
        
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