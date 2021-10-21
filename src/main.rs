use std::{
    alloc::Layout, fs::OpenOptions, os::unix::prelude::AsRawFd, path::PathBuf, str::FromStr,
};

use color_eyre::{
    eyre::{bail, eyre, Context, Result},
    Report,
};
use nix::{errno::Errno, request_code_none, request_code_read, sys::ioctl::ioctl_num_type};
use structopt::StructOpt;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    setup()?;
    let options = Options::from_args();
    run_command(&options)?;
    Ok(())
}

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
    #[structopt(long)]
    align: Option<usize>,
    #[structopt(long)]
    size: Option<usize>,
}

impl Options {
    fn layout(&self) -> Result<Layout> {
        let size = self
            .size
            .ok_or_else(|| eyre!("missing '--size' argument."))?;
        let align = self
            .align
            .ok_or_else(|| eyre!("missing '--align' argument."))?;
        Ok(Layout::from_size_align(size, align)?)
    }
}

fn setup() -> Result<()> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}

fn run_command(options: &Options) -> Result<()> {
    let file = OpenOptions::new()
        .read(options.read)
        .write(options.write)
        .open(&options.file)
        .map_err(|e| eyre!(e))
        .wrap_err_with(|| format!("Couldn't open {:?}", &options.file))?;
    let fd = file.as_raw_fd();
    match (options.read, options.write) {
        (false, false) => match options.r#type {
            None | Some(Type::None) => {
                let result = Errno::result(unsafe {libc::ioctl(
                        fd,
                        request_code_none!(options.ioctl_id, options.seq_no),
                    )})?;
                info!(%result);
                return Ok(());
            }
            Some(other) => bail!(
                "invalid type {:?} for a no-read, no-write ioctl: must be 'none' (defaults to 'none' if unspecified).",
                other
            ),
        },
        (true, false) => {
            let layout = options
                .layout()
                .wrap_err_with(|| "Must provide '--size' and '--align' when reading.")?;
            match options.r#type {
                None | Some(Type::Ptr) => {
                    let mut data = vec![0u8; layout.size() + layout.align()];
                    let offset = data.as_ptr().align_offset(layout.align());
                    let result = {
                        let data = data.as_mut_ptr();
                        let data = data.wrapping_add(offset);
                        Errno::result(unsafe {
                            libc::ioctl(fd, request_code_read!(options.ioctl_id, options.seq_no, layout.size()), data)
                        })?
                    };
                    info!(%result);
                },
                Some(Type::Buf) => unimplemented!(),
                Some(other) => bail!(
                    "invalid type {:?} for a read, no-write ioctl: must be 'ptr' or 'buf' (defaults to 'ptr' if unspecified).",
                    other
                ),
            }
        }
        _ => unimplemented!(),
    }
    Ok(())
}
