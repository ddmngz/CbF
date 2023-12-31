use anyhow::{Result,Context};
use bacon_sci::interp::lagrange;
use bacon_sci::polynomial::Polynomial;
use clap::Parser;
use std::str;
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
/// EXTENSION SIZE u8 | EXTENSION | CHUNK SIZE (in bytes) | NUMBER OF CHUNKS | POLYNOMIAL DEGREES
/// this means that the file extension can't be bigger than a u8
fn simple_cbf(input: &mut File, output: &mut File,ext:&str) -> Result<()> {
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
    //println!("{chunks} chunks of size {chunksize}, slice length {}",vecs[0].len());
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
    let bytes = ext.as_bytes();
    output.write(&bytes)?;
    //refactor later
    let keys: Vec<f64> = vecs.iter().map(bytestonum).collect::<Vec<f64>>();
    let xs: Vec<f64> = (0..keys.len()).map(|x| x as f64).collect();
    let coeffs = lagrange(&xs, &keys, 1e-6).unwrap().get_coefficients();
    println!("values of {:?} can be modeled with a polynomial with coefficients {:?}",keys,coeffs);
    output.write_u64::<LittleEndian>(chunksize as u64)?;
    output.write_u64::<LittleEndian>(chunks as u64)?;
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
    let ext = ext.to_str().expect("couldn't turn osstr into str");
    let (mut input,mut output) = open_files(args)?;
    simple_cbf(&mut input,&mut output,&ext)
}


/// EXTENSION SIZE u8 | EXTENSION | CHUNK SIZE (in bytes) | POLYNOMIALS
fn simple_decompress(input: &mut File,output: Option<PathBuf>) -> Result<()>{
    //let mut input_bytes:Vec<u8> = Vec::new();
    // get size of extension
    let mut e_size:[u8;8] = [0;8];
    input.read_exact(&mut e_size)?;
    let e_size:usize = e_size[0].try_into()?;
    // get file extension
    let mut extension:Vec<u8> = vec![0;e_size as usize];
    input.read(&mut extension[0..e_size])?;
    let extension = str::from_utf8(&extension)?;
    let chunksize = input.read_u64::<LittleEndian>()?;
    let number_of_chunks = input.read_u64::<LittleEndian>()?;
    let mut polys:Vec<f64> = Vec::new(); 
    println!("extension size: {e_size}, extension: {extension}, chunksize: {chunksize}, number of chunks: {number_of_chunks}");
    let mut buf = [0_u8;size_of::<f64>()];
    while input.read(&mut buf).is_ok_and(|x| x > 0) {
        polys.push(f64::from_le_bytes(buf));
    }
    let func = Polynomial::from_slice(&polys);
    //TODO: work for bigger chunks
    let mut new_file:Vec<u8> = Vec::new();
    for i in 0..number_of_chunks{
        let scalar = func.evaluate(i as f64).round() as u8;
        println!("f({i}) = {scalar}");
        new_file.push(scalar);
    }
    let mut output_file:File;
    match output{
        Some(path) => output_file = File::create(path)?,
        None => output_file = File::create(format!("output.{}",extension))?,
    }
    output_file.write_all(&new_file)?;
    Ok(())
}




fn decompress(args:Args) -> Result<()>{
    let mut input = File::open(args.input).expect("file {args.input} not found");
    simple_decompress(&mut input,args.output)?;
    Ok(())
}


fn main() {
    let args = Args::parse();
    let mut response = String::new();
    println!("c for compress, d for decompress:");
    std::io::stdin().read_line(&mut response).expect("goofy");
    match &response[..]{
        "c\n" => compress(args).expect("error"),
        "d\n" => decompress(args).expect("decompression error"),
        _ => println!("invalid character"),
    }
}
