# Compression by Function!
A little side project I'm doing to practice design, understand compression and a little bit of cryptography, but mostly for fun :). I don't really think anything I'm doing is novel or revolutionary, but I think a project is a good way to generate interest in a field and learn more!

## What am I trying to do?
The main principle that interested me was that, from my understanding, lossless compression algorithms take advantage of repetitions of ordered data, store each repetition once, and then optimally store the locations of each repetition. Instead, I want to try using interpolation and regression to encode the data as a function. In mathy terms, given chunks of data c, i want to make a function f(x) such that there is a set of values v that produce c at f(v).

## Cool,, so how's it going
I just got through the proof of concept stage, encoding and decoding Hello World using Lagrange polynomial interpolation. It's currently pretty inefficient, but I have a lot of plans to optimize it further. Ideally, I want to be able to compress arbitrary data, regardless of the way it's ordered, with relatively low overhead, but I doubt that I'm at a level where I can do that, so I'll see how it goes. Part of me wants to rename this to EbF (encoding by function), but I think keeping the name is a good motivator for optimizing this project.

## How do I use it?
This project is in such small scope, is currently in the proof of concept stage, and coming from someone with no real online presence, I doubt there is any usecase where this would be better than SHA-2 for cryptographic uses, Huffman coding for lossless compression, or DCT for lossy compression. If anything changes, I'll let you know, but for now this is for fun! 

## Todo
## Lagrange CBF
- [ ] Refactor to make next steps easier
- [ ] Placeholder for determining chunk size for SimpleCbf
- [ ] Create algorithm for produce the simplest lagrange interpolation (optimize for compression first (lower degee polynomial), then performance)
- [ ] Determine Optimal Chunk size & implement it
- [ ] Make Lagrange CBF lossless by reimplementing it

## Arbitrary Function CBF
- [ ] Create framework for encoding arbitrary functions rather than just polynomials
- [ ] Use this to explore which usecases work better for different functions

## Lossy CBF
- [ ] Use regression rather than interpolation + Float imprecision to see if that works for lossy compression
- [ ] Implement DCT into CBF Framework since it is really good

## Optimal CBF
- [ ] Based on Arbitrary Function CBF and Lossy CBF, make an algorithm that compresses based on whatever functions work the best in different usecases
