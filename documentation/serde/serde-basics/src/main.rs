// https://docs.rs/serde_json/latest/serde_json/

/// Tutorial Documentation:
/// https://medium.com/@itsuki.enjoy/detail-guide-to-serialization-and-deserialization-with-serde-in-rust-4fa70a6a8c4b


// Simple Struct of Student
#[derive(serde::Serialize, serde::Deserialize)]
struct Student {
    pub name: String,
    pub student_id: String,
}

// Serde Crate Documentation: 
// https://docs.rs/serde_json/latest/serde_json/
fn main() {
    println!("Serde Documentation:");
    let student = Student {
        name:"Greg".to_owned(), 
        student_id:"932670214".to_owned()
    };

    // convert to json 
    println!("JSON String of the Student Struct:");
    let student_json = serde_json::to_string(&student).unwrap();
    // convert to a student JSON
    println!("Student: {:?}", &student_json);

    println!("JSON String to Struct Student:");
    // deserialize it
    let mr_greg: Student = serde_json::from_str(&student_json).unwrap();
    println!("Student Name: {}", mr_greg.name);
    println!("Student ID: {}", mr_greg.student_id);

}
