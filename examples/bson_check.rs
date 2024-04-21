use bson::{SerializerOptions, Document};
use serde::{Serialize, Deserialize};

#[derive(Debug,Serialize,Deserialize)]
struct Bison {
    name: String,
    age: u16,
    place: String,
    phone: u16,
}


pub fn check_bson() {
    let i = 5;
    let bison = Bison {
            name: format!("Name {}", i),
            age: i as u16,
            place: format!("Place {}", i),
            phone: i as u16,
        };

    let options = SerializerOptions::builder().human_readable(false).build();
    let bson = bson::to_bson_with_options(&bison, options).unwrap();
    println!("{:?}", bson);

    // let mut doc = Document::new();
    // doc.insert("array".to_string(), bson);

//    let mut buf = Vec::new();
//    bson.to_writer(&mut buf).unwrap();
    match bson::to_vec(&bison) {
        Ok(buf) =>  std::fs::write("data.bson", buf).expect("Failed to create file"),
        Err(err) => panic!("Failed to serialized bison.\n\tError: {err:?}")
    }
}


pub fn check_bson_vec() {
    let mut bisons: Vec<Bison> = Vec::with_capacity(1000);
    for i in 1..3 {
        bisons.push(Bison {
            name: format!("Name {}", i),
            age: i as u16,
            place: format!("Place {}", i),
            phone: i as u16,
        });
    }

    let options = SerializerOptions::builder().human_readable(false).build();
    let bson = bson::to_bson_with_options(&bisons, options).unwrap();
    println!("{:?}", bson);

    let mut doc = Document::new();
    doc.insert("array".to_string(), bson);

    let mut buf = Vec::new();
    doc.to_writer(&mut buf).unwrap();

    std::fs::write("data.bson", buf).expect("Failed to create file");
}


pub fn main () {
    check_bson();
}