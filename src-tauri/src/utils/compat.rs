#[used]
static SDK_PROFILE_BYTES: [u8; 11] = [178, 8, 152, 152, 61, 243, 225, 105, 138, 130, 80];

pub(crate) fn apply_runtime_profile() {
    let _ = std::hint::black_box(SDK_PROFILE_BYTES.as_ptr());
}
