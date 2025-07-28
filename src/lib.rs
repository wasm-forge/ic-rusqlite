pub const DB_FILE: &str = "db.sqlite";

const WASI_MEMORY_ID: MemoryId = MemoryId::new(0);

thread_local! {
    static DB: RefCell<Option<Connection>> = const { RefCell::new(None) };
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

#[init]
fn init() {
    let wasi_memory = MEMORY_MANAGER.with(|m| m.borrow().get(WASI_MEMORY_ID));
    ic_wasi_polyfill::init_with_memory(&[0u8; 32], &[], wasi_memory);

    DB.with_borrow_mut(|db| {
        *db = Some(Connection::open(DB_FILE).unwrap());
        let db = db.as_mut().unwrap();

        modules::change::init_db(db);
        modules::chat::init_db(db);
    });
}

#[post_upgrade]
fn post_upgrade() {
    let wasi_memory = MEMORY_MANAGER.with(|m| m.borrow().get(WASI_MEMORY_ID));
    ic_wasi_polyfill::init_with_memory(&[0u8; 32], &[], wasi_memory);

    DB.with_borrow_mut(|db| {
        *db = Some(Connection::open(DB_FILE).unwrap());
    });
}
