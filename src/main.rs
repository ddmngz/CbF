use anyhow::Result;
use bacon_sci::interp::lagrange;
use bacon_sci::polynomial::Polynomial;
use clap::Parser;
use std::fs::File;
use std::io::prelude::*;
use std::io::{IoSliceMut,Cursor};
use std::path::{PathBuf,Path};
use std::ffi::{OsStr,OsString};
use std::mem::size_of;
use byteorder::{LittleEndian,ReadBytesExt,WriteBytesExt};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    /// file output (default is <name.cbf>)
    output: Option<PathBuf>,
}

static NO_VALS: usize = 256;
/// Compress a file via lagrange interpolation of (chunk_no,chunk_data) where chunk_data is the
/// numerical integer interprotation of chunk bytes 
/// the size of the chunk will optimize for minimum # of chunks that evaluate to a value that can
/// be encoded as a float
/// put in the following format
/// EXTENSION SIZE u8 | EXTENSION | CHUNK SIZE (in bytes) | POLYNOMIALS
/// this means that the file extension can't be bigger than a u8
fn simple_cbf(input: &mut File, output: &mut File,ext:&OsStr) -> Result<()> {
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
        input
            .read_vectored(&mut [IoSliceMut::new(&mut buf[0..chunksize])])
            .unwrap();
    }
    let bytestonum = |x: &Vec<u8>| {
        let mut num: f64 = 0.0;
        for (i, n) in x.iter().enumerate(){
            println!("{}",i);
            num += (*n as u64 * 256_u64.pow(i as u32)) as f64;
        }
        println!("num: {num}");
        num
    };
    //write string and size
    output.write_u64::<LittleEndian>(ext.len() as u64)?;
    let bytes = ext.as_encoded_bytes();
    output.write(&bytes)?;
    //refactor later
    let keys: Vec<f64> = vecs.iter().map(bytestonum).collect::<Vec<f64>>();
    let xs: Vec<f64> = (0..keys.len()).map(|x| x as f64).collect();
    let coeffs = lagrange(&xs, &keys, 1e-6).unwrap().get_coefficients();
    println!("values of {:?} can be modeled with a polynomial with coefficients {:?}",keys,coeffs);
    output.write_u64::<LittleEndian>(chunksize as u64)?;
    for coeff in coeffs {
        output.write_f64::<LittleEndian>(coeff)?;
    }
    Ok(())
}


fn open_files(args:Args) -> Result<(File,File)>{
    let output = if let Some(output) = args.output {
        File::create(output).expect("directory doens't exist")
    } else {
        File::create(String::from(args.input.file_stem().unwrap().to_str().unwrap()) + ".cbf")
            .expect("directory doesn't exist")
    };
    let input = File::open(args.input).expect("file {args.input} not found");
    Ok((input,output))
}

fn get_extension(s:&Path) -> OsString{
    match s.extension(){
        Some(ext) => ext.to_os_string(),
        None => OsString::new(),
    }
}

fn compress(args:Args) -> Result<()>{
    //get the file extension, open the files, and then do cbf 
    let ext = get_extension(&args.input);
    let (mut input,mut output) = open_files(args)?;
    simple_cbf(&mut input,&mut output,&ext)
}


/// EXTENSION SIZE u8 | EXTENSION | CHUNK SIZE (in bytes) | POLYNOMIALS
fn simple_decompress(input: &mut File,output: Option<&mut File>) -> Result<()>{
    let mut input_bytes:Vec<u8> = Vec::new();
    let e_size:u8 = 0;
    input.read(&mut[e_size])?;
    let mut extension:Vec<u8> = vec![0;e_size as usize];
    input.read_exact(&mut extension)?;
    let mut chunksize = [0_u8;size_of::<usize>()];
    input.read_exact(&mut chunksize)?;
    let chunksize = usize::from_le_bytes(chunksize);
    let mut polys:Vec<f64> = Vec::new(); 
    input.read_to_end(&mut input_bytes)? ;
    let mut buf = [0_u8;8];
    while input.read(&mut buf).is_ok_and(|x| x != 0) {
        polys.push(f64::from_le_bytes(buf));
    }
    println!("decompressed to {:?}",polys);
    Ok(())
}




fn decompress(args:Args) -> Result<()>{
    let (mut input,mut output) = open_files(args)?;
    Ok(())
}


fn main() {
    let args = Args::parse();
    compress(args).expect("error");
}
