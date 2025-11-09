use uuid::Uuid;
use waywe_uuid::TypeUuid;

#[derive(TypeUuid)]
#[uuid = "d4adfc76-f5f4-40b0-8e28-8a51a12f5e46"]
struct MyType;

fn main() {
    let uuid = Uuid::from_bytes(MyType::uuid());
    println!("{}", uuid);
}
