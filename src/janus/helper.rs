use rand::prelude::*;

// TODO: u64 not fit Javascript Number (i53)
pub fn rand_id() -> u64 {
    let mut rng = thread_rng();     // TODO: this may affect performances?
    loop {
        let n: u64 = rng.next_u32() as u64;
        if n != 0 {
            return n;
        }
    }
}
