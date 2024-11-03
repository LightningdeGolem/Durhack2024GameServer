use crate::OBJECTS;

pub trait MyObject: Send + Sync + 'static {
    fn tick(&mut self) {}
    fn x(&self) -> u64;
    fn y(&self) -> u64;
    fn img(&self) -> u64;
}

pub async fn object_tick() {
    loop {
        for (_, object) in OBJECTS.lock().await.iter_mut() {
            object.tick();
        }
    }
}
