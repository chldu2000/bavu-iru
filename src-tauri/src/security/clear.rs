use zeroize::{Zeroize, Zeroizing};

pub fn secure_clear(data: &mut [u8]) {
    data.iter_mut().for_each(|b| *b = 0);
}

pub fn secure_clear_string(s: &mut String) {
    if !s.is_empty() {
        let vec = unsafe { s.as_mut_vec() };
        vec.iter_mut().for_each(|b| *b = 0);
        unsafe { vec.set_len(0) };
    }
}

pub fn secure_clear_vec(v: &mut Vec<u8>) {
    v.iter_mut().for_each(|b| *b = 0);
    v.clear();
}

pub fn secure_clear_zeroizing<T: Zeroize + AsMut<[u8]>>(data: &mut Zeroizing<T>) {
    data.zeroize();
}