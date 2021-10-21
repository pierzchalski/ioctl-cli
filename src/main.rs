use std::{
    fs::{File, OpenOptions},
    os::unix::prelude::AsRawFd,
    path::PathBuf,
    str::FromStr,
};

use color_eyre::{
    eyre::{bail, Result},
    Report,
};
use nix::{request_code_none, sys::ioctl::ioctl_num_type};
use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialEq, Eq, Clone, Copy)]
enum Type {
    None,
    Int,
    Ptr,
    Buf,
}

impl FromStr for Type {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match &*s.to_lowercase() {
            "none" => Type::None,
            "int" => Type::Int,
            "ptr" => Type::Ptr,
            other => bail!("Unexpected string {}", other),
        })
    }
}

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    read: bool,
    #[structopt(short, long)]
    write: bool,
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,
    #[structopt(short, long, parse(try_from_str = hex::decode))]
    data: Option<std::vec::Vec<u8>>,
    #[structopt(short, long)]
    ioctl_id: ioctl_num_type,
    #[structopt(short, long)]
    seq_no: ioctl_num_type,
    #[structopt(short, long)]
    r#type: Option<Type>,
}

fn main() -> Result<()> {
    let options = Options::from_args();
    let file = OpenOptions::new()
        .read(options.read)
        .write(options.write)
        .open(&options.file)?;
    let fd = file.as_raw_fd();
    match (options.read, options.write) {
        (false, false) => match options.r#type {
            None | Some(Type::None) => {
                let result = unsafe {
                    nix::errno::Errno::result(libc::ioctl(
                        fd,
                        request_code_none!(options.ioctl_id, options.seq_no),
                    ))?
                };
                return Ok(());
            }
            Some(other) => bail!(
                "invalid type {:?} for a no-read, no-write ioctl: must be 'none' or unspecified.",
                other
            ),
        },
        (true, false) => match options.r#type {
            None | Some(Type::Ptr) =>
        }
        _ => unimplemented!(),
    }
    Ok(())
}
