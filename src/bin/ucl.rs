/*
 * ucl - command line compressor using libucl
 * Copyright (C) 2020-2021  BMW Group
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use clap::{Arg, Command};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use memmap::MmapMut;

use uclcli::{compress, compress_into_buffer, minimum_compression_buffer_size, ucl_init};


fn main() -> Result<()> {
    let matches = Command::new("ucl")
        .version("0.1")
        .author("Kjell Braden <kjell.braden@bmw.de>")
        .about("libucl (NRV) compressor")
	.arg(Arg::new("INPUT")
      .short('i')
      .long("input")
      .value_name("INPUT")
      .help("Sets the input file to use [defaults to stdin]"))
	.arg(Arg::new("OUTPUT")
      .short('o')
      .long("output")
      .value_name("OUTPUT")
      .help("Sets the output file to use [defaults to stdout]"))
	.get_matches();

    ucl_init();

    let mut input: Box<dyn Read> = match matches.get_one::<String>("INPUT") {
        Some(path) => Box::new(
            OpenOptions::new()
                .read(true)
                .open(&path)
                .context("could not open input file")?,
        ),
        None => Box::new(io::stdin()),
    };

    let mut inbuffer = Vec::new();
    input.read_to_end(&mut inbuffer)?;

    let out_size = minimum_compression_buffer_size(inbuffer.len());

    let output_filename = matches.get_one::<String>("OUTPUT");
    match output_filename {
        Some(path) => {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)
                .context("could not create output file")?;
            file.set_len(out_size as u64)
                .context("could not resize output file")?;

            let numbytes = unsafe {
                let mut mmap = MmapMut::map_mut(&file).context("failed to map output file")?;
                let nb =
                    compress_into_buffer(&inbuffer, &mut mmap).context("decompression failed")?;
                mmap.flush().context("failed to write output")?;
                nb
            };
            file.set_len(numbytes.into())
                .context("failed to truncate output file")?;
        }
        None => {
            let dst = compress(&inbuffer).context("decompression failed")?;
            io::stdout().write_all(&dst)?;
        }
    }

    Ok(())
}
