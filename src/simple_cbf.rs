use crate::cbf_framework::CompressionFn;
use anyhow::{anyhow, Result};
use bacon_sci::interp::lagrange;
use bacon_sci::polynomial::Polynomial;
use byteorder::{ReadBytesExt,LittleEndian};
use std::fs::File;
use std::io::prelude::*;
use std::io::IoSliceMut;
use std::mem::size_of;

/// Compression via Lagrange Interpolation of a dataset (n,b) where n = the chunknumber and b = the
/// value of chunk N
/// This is very inefficient, but is useful for proof of concept/understanding the problem

/// Format:
/// EXTENSION SIZE u8 | EXTENSION | CHUNK SIZE (in bytes) | NUMBER OF CHUNKS | POLYNOMIAL DEGREES

pub struct SimpleCbF {
    chunksize: u64,
    no_chunks: u64,
    extension: Option<String>,
}

impl SimpleCbF {
    pub fn new() -> Self {
        Self {
            chunksize: 0,
            no_chunks: 0,
            extension: None,
        }
    }
    fn encode_metadata(&mut self, input: &File, ext: &str) -> Result<()> {
        let filesize = input.metadata()?.len();
        // TODO: HOW TO DETERMINE CHUNK SIZE?
        let chunksize = 1;
        // round up?
        let chunks = filesize / chunksize;
        self.chunksize = chunksize;
        self.no_chunks = chunks;
        self.extension = Some(ext.to_owned());
        Ok(())
    }
    // find a way to write without taking ownership
    fn write_metadata(&mut self, output: &mut File) -> Result<()> {
        if let Some(ext) = &self.extension {
            let extension_size: u8 = ext.len().try_into()?;
            output.write(&[extension_size])?;
            output.write(ext.as_bytes())?;
            output.write(&self.chunksize.to_le_bytes())?;
            output.write(&self.no_chunks.to_le_bytes())?;
            Ok(())
        } else {
            Err(anyhow!("invalid"))
        }
    }
    fn encode_contents(&self, input: &mut File, output: &mut File) -> Result<()> {
        // helper closure :3
        let bytestonum = |x: &Vec<u8>| {
            let mut num: f64 = 0.0;
            for (i, n) in x.iter().enumerate() {
                println!("{}", i);
                num += (*n as u64 * 256_u64.pow(i as u32)) as f64;
            }
            println!("num: {num}");
            num
        };
        // read in chunks
        let mut buffers = vec![vec![0u8; self.chunksize as usize]; self.no_chunks as usize];
        let mut io_slice_buf: Vec<IoSliceMut> = buffers
            .iter_mut()
            .map(|x| IoSliceMut::new(&mut x[..]))
            .collect();
        input.read_vectored(&mut io_slice_buf[..])?;
        // create lagrange polynomial
        let keys: Vec<f64> = buffers.iter().map(bytestonum).collect::<Vec<f64>>();
        let xs: Vec<f64> = (0..keys.len()).map(|x| x as f64).collect();
        let coeffs = lagrange(&xs, &keys, 1e-6).unwrap().get_coefficients();
        let bytes: Vec<u8> = coeffs
            .iter()
            .flat_map(|x| (*x).to_le_bytes().to_vec())
            .collect();
        output.write(&bytes[..])?;
        Ok(())
    }

    /// EXTENSION SIZE u8 | EXTENSION | CHUNK SIZE (in bytes) | NUMBER OF CHUNKS | POLYNOMIAL DEGREES
    fn decode_metadata(&mut self, input: &mut File) -> Result<()> {
        let size: usize = input.read_u8()?.try_into()?;
        let mut extension = vec![0u8; size];
        input.read_exact(&mut extension[..])?;
        let extension = String::from_utf8(extension)?;
        //println!("extension is {}",extension);
        self.extension = Some(extension);
        self.chunksize = input.read_u64::<LittleEndian>()?;
        self.no_chunks = input.read_u64::<LittleEndian>()?;
        Ok(())
    }
    fn decode_contents(&mut self, input: &mut File, output: &mut File) -> Result<()> {
        // get polynomial
        let mut polys: Vec<f64> = Vec::new();
        let mut buf = [0_u8; size_of::<f64>()];
        while input.read(&mut buf).is_ok_and(|x| x > 0) {
            polys.push(f64::from_le_bytes(buf));
        }
        let func = Polynomial::from_slice(&polys);
        let mut new_file: Vec<u8> = Vec::new();
        // this currently doesn't work for larger chunksizes
        println!("{}",self.no_chunks);
        for i in 0..self.no_chunks {
            let scalar = func.evaluate(i as f64).round() as u8;
            println!("f({i}) = {scalar}");
            new_file.push(scalar);
        }
        output.write_all(&new_file)?;
        Ok(())
    }
}

impl CompressionFn for SimpleCbF {
    fn encode(&mut self, input: &mut File, extension: &str, output: &mut File) -> Result<()> {
        self.encode_metadata(&input, extension)?;
        self.write_metadata(output)?;
        self.encode_contents(input, output)?;
        Ok(())
    }
    fn decode(&mut self, input: &mut File, output: &mut File) -> Result<()> {
        self.decode_metadata(input)?;
        self.decode_contents(input, output)?;
        Ok(())
    }
}
