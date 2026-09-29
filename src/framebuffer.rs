// El render (main.rs) escribe directo a `buffer` en paralelo por fila, así
// que este struct solo necesita reservar el espacio.
pub struct Framebuffer {
    pub buffer: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            buffer: vec![0; width * height],
        }
    }
}
