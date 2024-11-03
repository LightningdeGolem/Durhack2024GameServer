use tokio::sync::Mutex;
pub static ICON_STORE: Mutex<Vec<Vec<u8>>> = Mutex::const_new(Vec::new());

pub async fn add_icon(data: Vec<u8>) -> u64 {
    let mut store = ICON_STORE.lock().await;
    store.push(data);
    (store.len() - 1) as u64
}
