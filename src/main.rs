mod cbf_framework;
mod simple_cbf;
use anyhow::Result;
use simple_cbf::SimpleCbF;
use cbf_framework::CompressionFn;
use clap::Parser;
use std::path::PathBuf;
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    /// file output (default is <name.cbf>)
    output: Option<PathBuf>,
}


fn get_extension(input:&PathBuf) -> String{
    input.extension().unwrap().to_string_lossy().into_owned()
}

fn encode(args:Args,funct:&mut impl CompressionFn) -> Result<()>{
    let extension = get_extension(&args.input);
    let mut input = File::open(args.input).expect("directory doesn't exist");
    let mut output = File::create(args.output.unwrap()).expect("directory doesn'te xist");

    funct.encode(&mut input, &extension, &mut output)?;
    Ok(())
}

fn decode(args:Args, funct:&mut impl CompressionFn)-> Result<()>{
    let mut input = File::open(args.input).expect("directory doesn't exist");
    let mut output = File::create(args.output.unwrap()).expect("directory doesn'te xist");
    println!("decoding");
    funct.decode(&mut input, &mut output)?;
    println!("done");
    Ok(())
}

fn main(){
    let args = Args::parse();
    let mut response = String::new();
    println!("e to encode, d to decode:");
    std::io::stdin().read_line(&mut response).expect("goofy");
    let mut simple:SimpleCbF = SimpleCbF::new();
    match &response[..]{
        "c\n" => encode(args,&mut simple).expect("error"),
        "d\n" => decode(args,&mut simple).expect("decompression error"),
        _ => println!("invalid character"),
    }
}
