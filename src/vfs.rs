use ic_cdk::stable::StableMemoryError;

use ic_stable_structures::{Memory, memory_manager::VirtualMemory};

use sqlite_vfs::{LockKind, OpenKind, OpenOptions, Vfs};
use std::io::{self, ErrorKind};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const WASM_PAGE_SIZE: u64 = 64 * 1024; // 64KB

pub struct VfsPages<M: Memory> {
    memory: VirtualMemory<M>,
    lock_state: Arc<Mutex<LockState>>,
}

#[derive(Debug, Default)]
struct LockState {
    read: usize,
    write: Option<bool>,
}

unsafe impl<M: Memory> Send for VfsConnection<M> {}
unsafe impl<M: Memory> Sync for VfsConnection<M> {}

pub struct VfsConnection<M: Memory> {
    memory: VirtualMemory<M>,
    lock_state: Arc<Mutex<LockState>>,
    lock: LockKind,
}

pub static DB_NAME: &str = ":memory:";

/// Gets capacity of the stable memory in bytes.
fn memory_capacity<M: Memory>(memory: &VirtualMemory<M>) -> u64 {
    memory.size() * WASM_PAGE_SIZE
}

/// Attempts to grow the memory by adding new pages.
fn memory_set_size<M: Memory>(
    memory: &mut VirtualMemory<M>,
    size: u64,
) -> Result<u64, StableMemoryError> {
    let pages_required = size.div_ceil(WASM_PAGE_SIZE);

    let current_pages = memory.size();
    if current_pages < pages_required {
        Ok(memory.grow(pages_required - current_pages) as u64)
    } else {
        Ok(current_pages)
    }
}

impl<M: Memory> VfsPages<M> {
    pub fn new(memory: VirtualMemory<M>) -> Self {
        let lock = LockState::default();

        VfsPages {
            memory,
            lock_state: Arc::new(Mutex::new(lock)),
        }
    }
}

unsafe impl<M: Memory> Send for VfsPages<M> {}
unsafe impl<M: Memory> Sync for VfsPages<M> {}

impl<M: Memory + Clone> Vfs for VfsPages<M> {
    type Handle = VfsConnection<M>;

    fn open(&self, file_name: &str, opts: OpenOptions) -> Result<Self::Handle, io::Error> {
        // only allow particular database name
        if file_name != DB_NAME {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                format!("Unexpected file name `{file_name}`; expected `{DB_NAME}`"),
            ));
        }

        // Only main databases supported right now (no journal, wal, temporary, ...)
        if opts.kind != OpenKind::MainDb {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "Only DB file type is supported!",
            ));
        }

        Ok(VfsConnection {
            memory: self.memory.clone(),
            lock_state: self.lock_state.clone(),
            lock: LockKind::None,
        })
    }

    fn delete(&self, _file_name: &str) -> Result<(), io::Error> {
        self.memory.write(0, &[0u8; 16]);
        Ok(())
    }

    fn exists(&self, file_name: &str) -> Result<bool, io::Error> {
        if file_name != DB_NAME {
            return Ok(false);
        }

        if memory_capacity(&self.memory) == 0 {
            return Ok(false);
        }

        let mut buf = [0u8; 16];
        self.memory.read(0, &mut buf);

        for x in buf {
            if x > 0 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn temporary_name(&self) -> String {
        String::from(DB_NAME)
    }

    fn random(&self, buffer: &mut [i8]) {
        use rand::rand_core::RngCore;

        ic_wasi_polyfill::RNG.with(|rng| {
            let buf: &mut [u8] = unsafe {
                std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len())
            };

            let mut rng = rng.borrow_mut();
            rng.fill_bytes(buf);
        });
    }

    fn sleep(&self, duration: Duration) -> Duration {
        let now = Instant::now();

        let ms = (duration.as_millis() as u32).max(1);

        std::thread::sleep(Duration::from_secs(ms.into()));

        now.elapsed()
    }
}

impl<M: Memory> sqlite_vfs::DatabaseHandle for VfsConnection<M> {
    type WalIndex = sqlite_vfs::WalDisabled;

    fn size(&self) -> Result<u64, io::Error> {
        Ok(self.memory.size())
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> Result<(), io::Error> {
        if self.memory.size() > 0 {
            self.memory.read(offset, buf);
        }
        Ok(())
    }

    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<(), io::Error> {
        //.. do we need to ensure the right file size here?
        memory_set_size(&mut self.memory, buf.len() as u64 + offset)
            .map(|_| ())
            .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))?;

        self.memory.write(offset, buf);
        Ok(())
    }

    fn sync(&mut self, _data_only: bool) -> Result<(), io::Error> {
        Ok(())
    }

    fn set_len(&mut self, size: u64) -> Result<(), io::Error> {
        memory_set_size(&mut self.memory, size)
            .map(|_| ())
            .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
    }

    fn lock(&mut self, lock: LockKind) -> Result<bool, io::Error> {
        let ok = Self::lock(self, lock);
        Ok(ok)
    }

    fn reserved(&mut self) -> Result<bool, io::Error> {
        Ok(Self::reserved(self))
    }

    fn current_lock(&self) -> Result<LockKind, io::Error> {
        Ok(self.lock)
    }

    fn wal_index(&self, _readonly: bool) -> Result<Self::WalIndex, io::Error> {
        Ok(sqlite_vfs::WalDisabled)
    }
}

impl<M: Memory> VfsConnection<M> {
    fn lock(&mut self, to: LockKind) -> bool {
        if self.lock == to {
            return true;
        }

        let mut lock_state = self.lock_state.lock().unwrap();

        match to {
            LockKind::None => {
                if self.lock == LockKind::Shared {
                    lock_state.read -= 1;
                } else if self.lock > LockKind::Shared {
                    lock_state.write = None;
                }
                self.lock = LockKind::None;
                true
            }

            LockKind::Shared => {
                if lock_state.write == Some(true) && self.lock <= LockKind::Shared {
                    return false;
                }

                lock_state.read += 1;
                if self.lock > LockKind::Shared {
                    lock_state.write = None;
                }
                self.lock = LockKind::Shared;
                true
            }

            LockKind::Reserved => {
                if lock_state.write.is_some() || self.lock != LockKind::Shared {
                    return false;
                }

                if self.lock == LockKind::Shared {
                    lock_state.read -= 1;
                }
                lock_state.write = Some(false);
                self.lock = LockKind::Reserved;
                true
            }

            LockKind::Pending => {
                // cannot be requested directly
                false
            }

            LockKind::Exclusive => {
                if lock_state.write.is_some() && self.lock <= LockKind::Shared {
                    return false;
                }

                if self.lock == LockKind::Shared {
                    lock_state.read -= 1;
                }

                lock_state.write = Some(true);
                if lock_state.read == 0 {
                    self.lock = LockKind::Exclusive;
                    true
                } else {
                    self.lock = LockKind::Pending;
                    false
                }
            }
        }
    }

    fn reserved(&self) -> bool {
        if self.lock > LockKind::Shared {
            return true;
        }

        let lock_state = self.lock_state.lock().unwrap();
        lock_state.write.is_some()
    }
}

impl<M: Memory> Drop for VfsConnection<M> {
    fn drop(&mut self) {
        if self.lock != LockKind::None {
            self.lock(LockKind::None);
        }
    }
}
