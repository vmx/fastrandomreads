use std::{mem, time::Instant};

use futures_lite::stream::StreamExt;
use glommio::{
    io::{
        DmaStreamReader, ImmutableFile, ImmutableFileBuilder, MergedBufferLimit,
        ReadAmplificationLimit,
    },
    LocalExecutor,
};
use pretty_bytes::converter;

const NODE_SIZE: usize = 32;
const OFFSETS_FILE: &str = "/tmp/parentcache/v28-sdr-parent-2aa9c77c3e58259481351cc4be2079cc71e1c9af39700866545c043bfa30fb42.cache";
const DATA_FILE: &str = "/tmp/random.data";

//// Basd on https://github.com/DataDog/glommio/blob/74c6c02183e87500214046c7f00ef9a2d27f0703/examples/storage.rs#L110
async fn read_file_to_memory(mut stream: DmaStreamReader, buffer_size: usize) -> Vec<u32> {
    let mut data = Vec::new();

    let start = Instant::now();
    loop {
        let buffer = stream.get_buffer_aligned(buffer_size as _).await.unwrap();
        data.extend_from_slice(&*buffer);
        if buffer.len() < buffer_size {
            break;
        }
    }
    stream.close().await.unwrap();
    let time = start.elapsed();
    println!(
        "Reading {} file to memory in {:#?}.",
        converter::convert(data.len() as _),
        time
    );

    // Convert from Vec<u8> to Vec<u32>.
    // Based on https://stackoverflow.com/questions/48308759/how-do-i-convert-a-vect-to-a-vecu-without-copying-the-vector/55081958#55081958
    unsafe {
        let fraction = mem::size_of::<u32>() / mem::size_of::<u8>();
        let mut data_u32 = std::mem::ManuallyDrop::new(data);
        Vec::from_raw_parts(
            data_u32.as_mut_ptr() as *mut _,
            data_u32.len() / fraction,
            data_u32.capacity() / fraction,
        )
    }
}

async fn read_at_random_offsets(file: ImmutableFile, offsets: Vec<u32>) {
    let max_buffer_size = 4096;
    let num_parallel_reads = 10;

    //GO ON HERE and also spawn multiple threads as https://github.com/DataDog/glommio/blob/74c6c02183e87500214046c7f00ef9a2d27f0703/examples/storage.rs#L267 does
    for chunk in offsets.chunks(num_parallel_reads) {
        let iovs: Vec<(u64, usize)> = chunk
            .iter()
            .map(|offset| (u64::from(*offset), NODE_SIZE))
            .collect();
        //println!("vmx: do a random many read: {:?}", iovs);
        file.read_many(
            futures_lite::stream::iter(iovs),
            MergedBufferLimit::Custom(max_buffer_size),
            ReadAmplificationLimit::NoAmplification,
        )
        .for_each(|_| {})
        .await;
    }
}

fn main() {
    println!("Hello, world!");

    //Keep the random offsets we want to read from in memory.
    //let mut read_offsets: Vec::<u8> = Vec::with_capacity(2 * 512 * 1024 * 1024);

    let executor = LocalExecutor::default();
    executor.run(async {
        let offsets_file = ImmutableFileBuilder::new(OFFSETS_FILE)
            .build_existing()
            .await
            .unwrap();
        //let offsets_reader = DmaStreamReaderBuilder::new(offsets_file).build();
        let offsets_reader = offsets_file.stream_reader().build();

        let buffer_size = 4096;
        //Keep the random offsets we want to read from in memory.
        let offsets_data = read_file_to_memory(offsets_reader, buffer_size).await;
        println!("vmx: read_offsets len: {}", offsets_data.len());

        let data_file = ImmutableFileBuilder::new(DATA_FILE)
            .build_existing()
            .await
            .unwrap();
        read_at_random_offsets(data_file, offsets_data).await;
    });
}
