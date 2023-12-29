use anyhow::Result;
use bacon_sci::interp::lagrange;
use clap::Parser;
use std::fs::File;
use std::io::prelude::*;
use std::io::IoSliceMut;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    /// file output (default is <name.cbf>)
    output: Option<PathBuf>,
}

static NO_VALS: usize = 256;
/// Compress a file, return a result for error checking
fn compress(input: &mut File, output: &mut File) -> Result<()> {
    let filesize: usize = input
        .metadata()
        .unwrap()
        .len()
        .try_into()
        .expect("file size bigger than usize");
    let mut chunksize = filesize / NO_VALS;
    if chunksize == 0 {
        chunksize = 1;
    }
    let chunks = (filesize / chunksize).try_into().unwrap();
    let mut vecs: Vec<Vec<u8>> = vec![vec![0;chunksize]; chunks];
    println!("{chunks} chunks of size {chunksize}, slice length {}",vecs[0].len());
    for buf in vecs.iter_mut() {
        println!("a");
        input
            .read_vectored(&mut [IoSliceMut::new(&mut buf[0..chunksize])])
            .unwrap();
    }
    let bytestonum = |x: &Vec<u8>| {
        let mut num: f64 = 0.0;
        for (i, n) in x.iter().enumerate() {
            num += n.pow(i as u32) as f64;
        }
        num
    };

    let keys: Vec<f64> = vecs.iter().map(bytestonum).collect::<Vec<f64>>();
    let xs: Vec<f64> = (0..keys.len()).map(|x| x as f64).collect();
    let coeffs = lagrange(&xs, &keys, 1e-6).unwrap().get_coefficients();
    output.write(&chunksize.to_le_bytes())?;
    for coeff in coeffs {
        output.write(&coeff.to_le_bytes())?;
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    let mut output = if let Some(output) = args.output {
        File::create(output).expect("directory doens't exist")
    } else {
        File::create(String::from(args.input.file_stem().unwrap().to_str().unwrap()) + ".cbf")
            .expect("directory doesn't exist")
    };
    let mut input = File::open(args.input).expect("file {args.input} not found");
    compress(&mut input,&mut output).expect("error");
}
