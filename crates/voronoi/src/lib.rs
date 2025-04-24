
use bevy::{math::{UVec2, Vec2}, reflect::Reflect};

pub mod geometry;
pub mod delaunay;

pub struct Diagram {
    pub sites: Vec<UVec2>,
    pub regions: Vec<Region>
}

pub struct Region {
    pub polygon: Polygon
}

impl Region {
    pub fn new(polygon: Polygon) -> Self {
        Self { polygon }
    }

    pub fn get_polygon(&self) -> &Polygon {
        &self.polygon
    }

    pub fn get_polygon_mut(&mut self) -> &mut Polygon {
        &mut self.polygon
    }

    pub fn get_vertices(&self) -> &Vec<Vec2> {
        self.polygon.get_vertices()
    }

    pub fn get_vertices_mut(&mut self) -> &mut Vec<Vec2> {
        self.polygon.get_vertices_mut()
    }

    pub fn get_area(&self) -> f32 {
        self.polygon.get_area()
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct Polygon {
    vertices: Vec<Vec2>,
}

impl Polygon {
    /// Creates a new polygon with the given vertices
    /// 
    /// # Panics
    /// Panics if the number of vertices is less than 3
    pub fn new(vertices: Vec<Vec2>) -> Self {
        assert!(vertices.len() >= 3);
        Self { vertices }
    }

    pub fn add_vertex(&mut self, vertex: Vec2) {
        self.vertices.push(vertex);
    }

    pub fn remove_vertex(&mut self, vertex: Vec2) {
        self.vertices.retain(|v| *v != vertex);
    }

    pub fn get_vertices(&self) -> &Vec<Vec2> {
        &self.vertices
    }

    pub fn get_vertices_mut(&mut self) -> &mut Vec<Vec2> {
        &mut self.vertices
    }

    pub fn get_vertex(&self, index: usize) -> Option<&Vec2> {
        self.vertices.get(index)
    }

    pub fn get_vertex_mut(&mut self, index: usize) -> Option<&mut Vec2> {
        self.vertices.get_mut(index)
    }

    #[inline]
    pub fn get_vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn get_double_area(&self) -> f32 {
        let len = self.vertices.len();
        let mut area = 0.0;
        for i in 0..len {
            let j = (i + 1) % len;
            area += self.vertices[i].x * self.vertices[j].y - self.vertices[j].x * self.vertices[i].y;
        }
        area
    }

    #[inline]
    pub fn get_area(&self) -> f32 {
        self.get_double_area() * 0.5
    }

    pub fn get_perimeter(&self) -> f32 {
        let len = self.vertices.len();
        let mut perimeter = 0.0;
        for i in 0..len {
            let j = (i + 1) % len;
            perimeter += (self.vertices[i] - self.vertices[j]).length();
        }
        perimeter
    }

    pub fn get_centroid(&self) -> Vec2 {
        let len = self.vertices.len();
        let area = self.get_area();
        let mut centroid = Vec2::ZERO;
        for i in 0..len {
            let j = (i + 1) % len;
            let a = self.vertices[i].x * self.vertices[j].y - self.vertices[j].x * self.vertices[i].y;
            centroid += (self.vertices[i] + self.vertices[j]) * a;
        }
        centroid *= 1.0 / (6.0 * area);
        centroid
    }

    pub fn get_bounding_box(&self) -> AABB {
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);
        for vertex in &self.vertices {
            min = min.min(*vertex);
            max = max.max(*vertex);
        }
        (min, max)
    }

    
    pub fn contains_point(&self, point: Vec2) -> bool {
        if quick_check_bouindary_collision(point, self.get_bounding_box()).is_none() {
            return false;
        }
        let mut inside = false;
        let len = self.vertices.len();
        for i in 0..len {
            let j = (i + 1) % len;
            if ((self.vertices[i].y > point.y) != (self.vertices[j].y > point.y)) &&
                (point.x < (self.vertices[j].x - self.vertices[i].x) * (point.y - self.vertices[i].y) / (self.vertices[j].y - self.vertices[i].y) + self.vertices[i].x) {
                inside = !inside;
            }
        }
        inside
    }
}

type AABB = (Vec2, Vec2);

enum CollisionType {
    Inside = 1,
}

#[inline(always)]
fn quick_check_bouindary_collision(point : Vec2, aabb: AABB) -> Option<CollisionType> {
    if point.x < aabb.0.x || point.x > aabb.1.x || point.y < aabb.0.y || point.y > aabb.1.y {
        return None;
    }
    Some(CollisionType::Inside)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_centroid_regular_polygon() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        let polygon = Polygon::new(vertices.clone());
        
        let centroid = polygon.get_centroid();
        assert_eq!(centroid, Vec2::new(0.5, 0.5));
    }

    #[test]
    fn test_get_centroid_irregular_polygon() {
        let vertices = vec![
            Vec2::new(-3.0, -3.0),
            Vec2::new(3.0, -3.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(-3.0, 3.0),
        ];
        let polygon = Polygon::new(vertices.clone());
        
        let centroid = polygon.get_centroid();
        assert!((centroid.x.abs() < 1e-6).into() && (centroid.y.abs() < 1e-6).into());
    }

    #[test]
    fn test_get_centroid_convex_polygon() {
        let vertices = vec![
            Vec2::new(-2.0, -2.0),
            Vec2::new(2.0, -2.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(-2.0, 2.0),
        ];
        let polygon = Polygon::new(vertices.clone());
        
        let centroid = polygon.get_centroid();
        assert_eq!(centroid, Vec2::ZERO);
    }

    #[test]
    fn test_get_centroid_degenerate_polygon() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];
        let polygon = Polygon::new(vertices);
        
        let centroid = polygon.get_centroid();
        assert!(centroid.is_nan());
    }

    #[test]
    fn test_contains_point_convex_polygon() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        let polygon = Polygon::new(vertices.clone());
        
        assert!(polygon.contains_point(Vec2::new(0.5, 0.5)));
        assert!(!polygon.contains_point(Vec2::new(2.0, 2.0)));
    }

    #[test]
    fn test_contains_point_on_edge() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        let polygon = Polygon::new(vertices.clone());
        
        assert!(polygon.contains_point(Vec2::new(0.5, 0.5))); // Inside the polygon
        assert!(!polygon.contains_point(Vec2::new(1.5, 0.5))); // On an edge
        assert!(!polygon.contains_point(Vec2::new(1.5, 1.5))); // Outside the polygon
    }

    #[test]
    fn test_contains_point_degenerate_polygon() {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];
        let polygon = Polygon::new(vertices);
        
        assert!(!polygon.contains_point(Vec2::new(0.0, 0.0))); // Degenerate polygon
    }
}
