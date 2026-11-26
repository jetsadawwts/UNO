use crate::ports::{ProjectStore, SessionProbe};

/// Fingerprint of everything the page shows. The live page polls it and refreshes when it changes.
/// FNV-1a over: manual stamp, data stamp, open sessions — the same inputs and order as v0.5.12.
pub fn signature(store: &impl ProjectStore, manual_stamp: &str, sessions: &impl SessionProbe) -> String {
    let mut sig = String::from(manual_stamp);
    sig += &store.stamp();
    if let Some(s) = sessions.open_sessions() {
        sig += &format!("{s:?}");
    }
    let mut h: u64 = 1469598103934665603;
    for b in sig.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}
