mod ryu_mod {
    include!("mod.rs");
}
use ryu_mod::ryu_f32_mod;

fn main() {
    println!("{}", float32_to_string(1.5_f32));
}
