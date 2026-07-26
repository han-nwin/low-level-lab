mod bit_writer;
use std::io;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // == TASK 1
    // read to a Vec<u8>
    let content = tokio::fs::read("image.jpeg").await?;
    println!("Data Length: {}", &content.len());

    // write to file
    // tokio::fs::write("out.jpeg", content).await?;

    // TASK 2: Bitwriter BitReader
    //
    let mut writer = bit_writer::Writer::new();
    writer.write(89, 8)?;
    writer.write(10000, 8)?;
    println!("writer.packed: {:?}", writer.packed);

    Ok(())
}
