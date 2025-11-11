use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    ptr::NonNull,
};

mod ffi;

pub struct Connection {
    inner: NonNull<ffi::sqlite3>,
}

impl Connection {
    pub fn open(path: &str) -> Option<Self> {
        let mut inner: *mut ffi::sqlite3 = std::ptr::null_mut();
        unsafe {
            ffi::sqlite3_open(CString::new(path).ok()?.as_ptr(), &mut inner);
        }

        Some(Self {
            inner: NonNull::new(inner)?,
        })
    }

    pub fn prepare<'a>(&'a self, sql: &str) -> Result<Statement<'a>, u32> {
        let mut statement: *mut ffi::sqlite3_stmt = std::ptr::null_mut();
        let result = unsafe {
            ffi::sqlite3_prepare_v2(
                self.inner.as_ptr(),
                sql.as_ptr() as *const i8,
                sql.len() as i32,
                &mut statement,
                std::ptr::null_mut(),
            )
        };

        Ok(Statement {
            inner: NonNull::new(statement).ok_or(result as u32)?,
            _conn: PhantomData,
        })
    }

    pub fn execute<'a, S: Into<String>>(&'a self, sql: S) -> Result<(), u32> {
        let cstring = CString::new(sql.into()).map_err(|_| ffi::SQLITE_ERROR)?;
        let result = unsafe {
            ffi::sqlite3_exec(
                self.inner.as_ptr(),
                cstring.as_ptr(),
                None,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } as u32;

        if result == ffi::SQLITE_OK {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn error_message(&self) -> Option<&str> {
        unsafe {
            let ptr = ffi::sqlite3_errmsg(self.inner.as_ptr());
            if ptr.is_null() {
                return None;
            }

            let cstr = CStr::from_ptr(ptr);
            cstr.to_str().ok()
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_close_v2(self.inner.as_mut());
        }
    }
}

pub struct Statement<'conn> {
    inner: NonNull<ffi::sqlite3_stmt>,
    _conn: PhantomData<&'conn Connection>,
}

impl<'conn> Statement<'conn> {
    pub fn bind_text<'a>(&'a self, parameter: u32, text: &'a str) -> Result<(), u32> {
        let result = unsafe {
            ffi::sqlite3_bind_text(
                self.inner.as_ptr(),
                parameter as i32,
                text.as_ptr() as *const i8,
                text.len().try_into().ok().ok_or(ffi::SQLITE_RANGE)?,
                None,
            )
        } as u32;

        if result == ffi::SQLITE_OK {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn bind_blob<'a>(&'a self, parameter: u32, blob: &'a [u8]) -> Result<(), u32> {
        let result = unsafe {
            ffi::sqlite3_bind_text(
                self.inner.as_ptr(),
                parameter as i32,
                blob.as_ptr() as *const i8,
                blob.len().try_into().ok().ok_or(ffi::SQLITE_RANGE)?,
                None,
            )
        } as u32;

        if result == ffi::SQLITE_OK {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn execute(self) -> Result<(), u32> {
        while let Step::Row(_) = self.step()? {}
        Ok(())
    }

    pub fn step<'a>(&'a self) -> Result<Step<'a, 'conn>, u32> {
        let result = unsafe { ffi::sqlite3_step(self.inner.as_ptr()) };

        match result as u32 {
            ffi::SQLITE_ROW => {
                let columns = unsafe { ffi::sqlite3_column_count(self.inner.as_ptr()) } as u32;

                if columns == 0 {
                    return Err(ffi::SQLITE_RANGE);
                }

                Ok(Step::Row(Row {
                    statement: self,
                    columns,
                }))
            }
            ffi::SQLITE_DONE => Ok(Step::Done),
            _ => Err(result as u32),
        }
    }

    pub fn rows(self) -> Rows<'conn> {
        Rows { statement: self }
    }
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_finalize(self.inner.as_ptr());
        }
    }
}

pub enum Step<'a, 'conn> {
    Row(Row<'a, 'conn>),
    Done,
}

pub struct Row<'a, 'conn> {
    columns: u32,
    statement: &'a Statement<'conn>,
}

impl Row<'_, '_> {
    pub fn column_text<'a>(&'a self, column: u32) -> Option<&'a str> {
        if column >= self.columns {
            return None;
        }

        unsafe {
            let ptr = ffi::sqlite3_column_text(self.statement.inner.as_ptr(), column as i32);
            let len = ffi::sqlite3_column_bytes(self.statement.inner.as_ptr(), column as i32);

            if ptr.is_null() {
                return None;
            }

            let slice = std::slice::from_raw_parts(ptr, len as usize);
            str::from_utf8(slice).ok()
        }
    }

    pub fn column_blob<'a>(&'a self, column: u32) -> Option<&'a [u8]> {
        if column >= self.columns {
            return None;
        }

        unsafe {
            let ptr = ffi::sqlite3_column_text(self.statement.inner.as_ptr(), column as i32);
            let len = ffi::sqlite3_column_bytes(self.statement.inner.as_ptr(), column as i32);

            if ptr.is_null() {
                return None;
            }

            Some(std::slice::from_raw_parts(ptr, len as usize))
        }
    }
}

pub struct Rows<'conn> {
    statement: Statement<'conn>,
}

impl<'a, 'conn> Iterator for &'a Rows<'conn> {
    type Item = Row<'a, 'conn>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.statement.step() {
            Ok(Step::Row(row)) => Some(row),
            _ => None,
        }
    }
}
