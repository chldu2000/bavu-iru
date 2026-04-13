use zeroize::Zeroize;

/// Securely clear a byte slice from memory.
pub fn secure_clear(data: &mut [u8]) {
	data.zeroize();
}
