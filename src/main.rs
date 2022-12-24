use passphrase::gen_passphrase;

mod passphrase;

fn main() {
    let pass = gen_passphrase(8);
    println!("pp -> {}", pass);
}
