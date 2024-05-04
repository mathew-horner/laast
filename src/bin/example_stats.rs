use laast_rs::{examples, similarity};

#[tokio::main]
async fn main() {
    let example_laasts = examples::read("hello-world").await.unwrap();
    let example_stats = similarity::calculate(&example_laasts);
    println!("{example_stats}");
}
