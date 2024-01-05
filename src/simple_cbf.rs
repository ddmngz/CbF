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
    chunksize: usize,
    no_chunks: usize,
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
        // optimize greater chunksize because less chunks = simpler polynomial
        let chunksize = (filesize as f64/size_of::<u32>() as f64).ceil() as usize;
        // round up?
        let chunks = (filesize / chunksize as u64) as usize;
        self.chunksize = chunksize;
        self.no_chunks = chunks;
        self.extension = Some(ext.to_owned());
        Ok(())
    }
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
        // anonymous function to interpret the bits as a number
        let bytestonum = |x: &Vec<u8>| {
            println!("interpreting the following as one little endian number");
            for i in x{
                println!("{:b}",*i);
            }
            if x.len() > 4{
                panic!("encoded wrong (chunk too big");
            }
            let mut bytes = [0;4];
            bytes[..x.len()].copy_from_slice(&x[..x.len()]);
            let val = u32::from_le_bytes(bytes);
            println!("got {val}");
            println!("as a float is {}", val as f64);
            val as f64
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
        println!("chunksize {}",self.chunksize);
        println!("values: {:?}",keys);
        let xs: Vec<f64> = (0..keys.len()).map(|x| x as f64).collect();
        let coeffs = lagrange(&xs, &keys, 1e-20).unwrap().get_coefficients();
        println!("coeffs: {:?}",coeffs);
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
        self.chunksize = input.read_u64::<LittleEndian>()?.try_into()?;
        self.no_chunks = input.read_u64::<LittleEndian>()?.try_into()?;
        Ok(())
    }
    fn decode_contents(&mut self, input: &mut File, output: &mut File) -> Result<()> {
        // get polynomial
        let mut polys: Vec<f64> = Vec::new();
        let mut buf = [0_u8; size_of::<f64>()];
        while input.read(&mut buf).is_ok_and(|x| x > 0) {
            polys.push(f64::from_le_bytes(buf));
        }
        println!("coefficients: {:?}",polys);
        let func = Polynomial::from_slice(&polys);
        let mut new_file: Vec<u8> = Vec::new();
        for i in 0..self.no_chunks {
            let scalar = func.evaluate(i as f64).round() as u32;
            println!("f({i}) = {scalar}");
            new_file.extend(scalar.to_le_bytes());
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
