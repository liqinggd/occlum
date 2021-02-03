use super::*;

pub fn do_posix_fallocate(fd: FileDesc, offset: u64, len: u64) -> Result<()> {
    debug!(
        "posix_fallocate: fd: {}, offset: {}, len: {}",
        fd, offset, len
    );
    let file_ref = current!().file(fd)?;
    let cur_len = file_ref.metadata()?.size as u64;
    let new_len = offset
        .checked_add(len)
        .ok_or_else(|| errno!(EFBIG, "new length exceeds the maximum file size"))?;
    if new_len > cur_len {
        file_ref.set_len(new_len)?;
    }
    Ok(())
}
