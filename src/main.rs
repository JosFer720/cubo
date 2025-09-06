use minifb::{Key, Window, WindowOptions};
use nalgebra::{Point3, Vector3, Matrix3};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Point3<f32>, ray_direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>)>;
}

pub struct Cube {
    pub center: Point3<f32>,
    pub size: f32,
    pub rotation: Matrix3<f32>,
}

impl Cube {
    fn new(center: Point3<f32>, size: f32) -> Self {
        Cube {
            center,
            size,
            rotation: Matrix3::identity(),
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Point3<f32>, ray_direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>)> {
        let inv_rotation = self.rotation.transpose();
        let local_origin = self.center + inv_rotation * (ray_origin - self.center);
        let local_direction = inv_rotation * ray_direction;
        
        let half_size = self.size / 2.0;
        let min = self.center - Vector3::new(half_size, half_size, half_size);
        let max = self.center + Vector3::new(half_size, half_size, half_size);
        
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        let mut normal = Vector3::new(0.0, 0.0, 0.0);
        
        for i in 0..3 {
            if local_direction[i].abs() < 1e-6 {
                if local_origin[i] < min[i] || local_origin[i] > max[i] {
                    return None;
                }
            } else {
                let t1 = (min[i] - local_origin[i]) / local_direction[i];
                let t2 = (max[i] - local_origin[i]) / local_direction[i];
                
                let (t_near, t_far) = if t1 < t2 {
                    (t1, t2)
                } else {
                    (t2, t1)
                };
                
                if t_near > t_min {
                    t_min = t_near;
                    normal = Vector3::zeros();
                    normal[i] = if local_direction[i] > 0.0 { -1.0 } else { 1.0 };
                }
                
                t_max = t_max.min(t_far);
                
                if t_min > t_max {
                    return None;
                }
            }
        }
        
        if t_min < 0.001 {
            return None;
        }
        
        let world_normal = self.rotation * normal;
        Some((t_min, world_normal.normalize()))
    }
}

pub struct Plane {
    pub point: Point3<f32>,
    pub normal: Vector3<f32>,
}

impl RayIntersect for Plane {
    fn ray_intersect(&self, ray_origin: &Point3<f32>, ray_direction: &Vector3<f32>) -> Option<(f32, Vector3<f32>)> {
        let denom = self.normal.dot(ray_direction);
        
        if denom.abs() > 1e-6 {
            let t = (self.point - ray_origin).dot(&self.normal) / denom;
            if t >= 0.001 {
                return Some((t, self.normal));
            }
        }
        None
    }
}

fn is_in_shadow(point: &Point3<f32>, light_pos: &Point3<f32>, cube: &Cube) -> bool {
    let to_light = light_pos - point;
    let distance_to_light = to_light.magnitude();
    let light_dir = to_light.normalize();
    
    let shadow_origin = point + Vector3::new(0.0, 0.001, 0.0);
    
    if let Some((t, _)) = cube.ray_intersect(&shadow_origin, &light_dir) {
        if t > 0.0 && t < distance_to_light {
            return true;
        }
    }
    false
}

pub fn cast_ray(ray_origin: &Point3<f32>, ray_direction: &Vector3<f32>, cube: &Cube, plane: &Plane, light_pos: &Point3<f32>) -> u32 {
    let mut closest_t = f32::INFINITY;
    let mut hit_normal = Vector3::new(0.0, 0.0, 0.0);
    let mut hit_object = 0;
    
    if let Some((t, normal)) = cube.ray_intersect(ray_origin, ray_direction) {
        if t < closest_t {
            closest_t = t;
            hit_normal = normal;
            hit_object = 1;
        }
    }
    
    if let Some((t, normal)) = plane.ray_intersect(ray_origin, ray_direction) {
        if t < closest_t {
            closest_t = t;
            hit_normal = normal;
            hit_object = 2;
        }
    }
    
    match hit_object {
        1 => {
            let hit_point = ray_origin + ray_direction * closest_t;
            let light_dir = (light_pos - hit_point).normalize();
            
            let diffuse = hit_normal.dot(&light_dir).max(0.0);
            let ambient = 0.2;
            let intensity = (ambient + diffuse * 0.8).min(1.0);
            
            let base_r = 0.3;
            let base_g = 0.7;
            let base_b = 0.9;
            
            let r = (base_r * intensity * 255.0) as u32;
            let g = (base_g * intensity * 255.0) as u32;
            let b = (base_b * intensity * 255.0) as u32;
            
            0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
        },
        2 => {
            let hit_point = ray_origin + ray_direction * closest_t;
            
            let in_shadow = is_in_shadow(&hit_point, light_pos, cube);
            
            let scale = 2.0;
            let checker = ((hit_point.x / scale).floor() as i32 + (hit_point.z / scale).floor() as i32) % 2 == 0;
            
            let base_color = if checker {
                (0.8, 0.8, 0.8)
            } else {
                (0.3, 0.3, 0.3)
            };
            
            let intensity = if in_shadow {
                0.3
            } else {
                let light_dir = (light_pos - hit_point).normalize();
                let diffuse = hit_normal.dot(&light_dir).max(0.0);
                let ambient = 0.4;
                (ambient + diffuse * 0.6).min(1.0)
            };
            
            let r = (base_color.0 * intensity * 255.0) as u32;
            let g = (base_color.1 * intensity * 255.0) as u32;
            let b = (base_color.2 * intensity * 255.0) as u32;
            
            0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
        },
        _ => {
            let gradient = (ray_direction.y + 0.5).max(0.0).min(1.0);
            let r = (20.0 + gradient * 30.0) as u32;
            let g = (25.0 + gradient * 35.0) as u32;
            let b = (35.0 + gradient * 45.0) as u32;
            0xFF000000 | (r << 16) | (g << 8) | b
        }
    }
}

struct Scene {
    cube: Cube,
    plane: Plane,
    light_pos: Point3<f32>,
    camera_pos: Point3<f32>,
    camera_target: Point3<f32>,
}

impl Scene {
    fn new() -> Self {
        Scene {
            cube: Cube::new(Point3::new(0.0, 1.0, 0.0), 2.0),
            plane: Plane {
                point: Point3::new(0.0, 0.0, 0.0),
                normal: Vector3::new(0.0, 1.0, 0.0),
            },
            light_pos: Point3::new(-3.0, 8.0, -3.0),
            camera_pos: Point3::new(5.0, 8.0, 5.0),
            camera_target: Point3::new(0.0, 0.0, 0.0),
        }
    }
    
    fn move_camera(&mut self, delta: Vector3<f32>) {
        self.camera_pos += delta;
    }
    
    fn render(&self, buffer: &mut Vec<u32>) {
        let fov = std::f32::consts::PI / 3.0;
        let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
        
        let forward = (self.camera_target - self.camera_pos).normalize();
        let right = forward.cross(&Vector3::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(&forward);
        
        for j in 0..HEIGHT {
            for i in 0..WIDTH {
                let x = (2.0 * (i as f32 + 0.5) / WIDTH as f32 - 1.0) * (fov / 2.0).tan() * aspect_ratio;
                let y = -(2.0 * (j as f32 + 0.5) / HEIGHT as f32 - 1.0) * (fov / 2.0).tan();
                
                let ray_direction = (forward + right * x + up * y).normalize();
                
                buffer[j * WIDTH + i] = cast_ray(&self.camera_pos, &ray_direction, &self.cube, &self.plane, &self.light_pos);
            }
        }
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    
    let mut window = Window::new(
        "Cubo",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });
    
    window.set_target_fps(60);
    
    let mut scene = Scene::new();
    let move_speed = 0.2;
    
    println!("==== CONTROLES ====");
    println!("A/S/W/D - Mover cámara");
    println!("Q/E - Subir/bajar cámara");
    println!("ESC - Salir");
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
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
        
        scene.render(&mut buffer);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}