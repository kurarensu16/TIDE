//! Windows Job Object abstraction for guaranteed process-tree termination.
//!
//! When an agent or child terminal is spawned, its process is assigned to a Job Object
//! configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. When TIDE stops the session
//! or drops the Job Object, Windows immediately terminates the entire process tree,
//! preventing orphaned compilers, dev servers, or test runners from locking worktree files.

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
    JobObjectExtendedLimitInformation, TerminateJobObject,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
#[cfg(windows)]
use std::mem::size_of;
#[cfg(windows)]
use std::os::windows::io::RawHandle;

pub struct JobObject {
    #[cfg(windows)]
    handle: HANDLE,
}

// Safety: Windows Job Object handles can be safely moved across threads.
unsafe impl Send for JobObject {}
unsafe impl Sync for JobObject {}

impl JobObject {
    /// Creates a new Job Object configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
    pub fn new() -> anyhow::Result<Self> {
        #[cfg(windows)]
        {
            let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                return Err(anyhow::anyhow!("Failed to create Windows Job Object"));
            }

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let res = unsafe {
                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };

            if res == 0 {
                unsafe { CloseHandle(handle) };
                return Err(anyhow::anyhow!("Failed to configure JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE"));
            }

            Ok(Self { handle })
        }
        #[cfg(not(windows))]
        {
            Ok(Self {})
        }
    }

    /// Assigns a running process to this Job Object.
    pub fn assign_process(&self, _process_handle: RawHandle) -> anyhow::Result<()> {
        #[cfg(windows)]
        {
            let res = unsafe { AssignProcessToJobObject(self.handle, _process_handle as HANDLE) };
            if res == 0 {
                return Err(anyhow::anyhow!("Failed to assign process to Job Object"));
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    /// Terminate all child processes assigned to this Job Object.
    pub fn terminate(&self) -> bool {
        #[cfg(windows)]
        {
            if !self.handle.is_null() && self.handle != INVALID_HANDLE_VALUE {
                unsafe { TerminateJobObject(self.handle, 1) != 0 }
            } else {
                false
            }
        }
        #[cfg(not(windows))]
        {
            true
        }
    }
}

impl Drop for JobObject {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            if !self.handle.is_null() && self.handle != INVALID_HANDLE_VALUE {
                unsafe {
                    CloseHandle(self.handle);
                }
            }
        }
    }
}
