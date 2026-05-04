pub fn title(text: &str) {
    println!("\n=== {text} ===\n");
}

pub fn ok(text: &str) {
    println!("✅ {text}");
}

pub fn warn(text: &str) {
    println!("⚠️  {text}");
}

pub fn fail(text: &str) {
    println!("❌ {text}");
}

pub fn line(text: &str) {
    println!("   {text}");
}

pub fn kv(key: &str, value: &str) {
    println!("   {key}: {value}");
}

pub fn solscan_devnet(signature: &str) {
    line(&format!(
        "Explorer: https://solscan.io/tx/{signature}?cluster=devnet"
    ));
}

pub fn tx_success(signature: &str) {
    ok("Transaction confirmed on Solana devnet");
    kv("Signature", signature);
    solscan_devnet(signature);
}
