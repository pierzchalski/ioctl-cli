use std::{fs::OpenOptions, os::unix::prelude::AsRawFd, path::Path};

use color_eyre::{
    eyre::{bail, eyre, Context},
    Result,
};
use libc::ioctl;
use nix::{ioctl_none, request_code_none, request_code_read, sys::ioctl};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::{
    c_types::{Ptr, Type},
    c_values::Value,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct IoctlDef {
    #[serde(default = "make_false")]
    read: bool,
    #[serde(default = "make_false")]
    write: bool,
    data_type: Option<Type>,
    ioctl_number: ioctl::ioctl_num_type,
    ioctl_type: ioctl::ioctl_num_type,
}

fn make_false() -> bool {
    false
}

impl IoctlDef {
    pub fn do_ioctl(&self, path: &Path, value: Option<Value>) -> Result<(i32, Option<Value>)> {
        let file = OpenOptions::new()
            .read(self.read)
            .write(self.write)
            .create(false)
            .open(path)
            .map_err(|e| eyre!(e))
            .wrap_err_with(|| format!("path: {:?}", path))?;
        let fd = file.as_raw_fd();
        match (self.read, self.write, &self.data_type) {
            (false, false, Some(_)) => {
                bail!("IOCTLs without input or output can't have an associated data type")
            }
            (false, false, None) => {
                let result = unsafe { ioctl(fd, ioctl_none!()) };
            }
        }
        if (self.read || self.write) && self.data_type.is_some() {}
        todo!()
    }

    fn ioctl_request(&self) -> Result<ioctl::ioctl_num_type> {
        Ok(match (self.read, self.write, &self.data_type) {
            (false, false, None) => request_code_none!(self.ioctl_number, self.ioctl_type),
            (
                true,
                false,
                Some(Type::Ptr(Ptr {
                    deref_target: target,
                })),
            ) => request_code_read!(),
        })
    }
}
