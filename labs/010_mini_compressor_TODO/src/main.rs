mod bit_reader;
mod bit_writer;
use std::io;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // == TASK 1
    // read to a Vec<u8>
    let content = tokio::fs::read("image.jpeg").await?;
    println!("Data Length: {}", content.len());

    // write to file
    // tokio::fs::write("out.jpeg", content).await?;

    // TASK 2: Bitwriter BitReader
    //
    let mut writer = bit_writer::Writer::new();
    writer.write(9764609, 20)?; // 1001[0100 11111111 00000001]
    // writer.write(10000, 8)?;
    println!("writer.packed: {:?}", writer.packed);

    let packed = writer.finalize()?;
    println!("After finalize: writer.packed: {:?}", packed);

    Ok(())
}
