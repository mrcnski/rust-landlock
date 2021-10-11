use crate::{uapi, AccessFs, BitFlags, CompatState, Compatibility, TryCompat, ABI};
use libc::close;
use std::io::Error;
use std::mem::size_of_val;
use std::os::unix::io::RawFd;

#[cfg(test)]
use crate::*;

// Public interface without methods and which is impossible to implement outside this crate.
pub trait Rule: PrivateRule {}

pub trait PrivateRule: TryCompat {
    fn as_ptr(&self) -> *const libc::c_void;
    fn get_type_id(&self) -> uapi::landlock_rule_type;
    fn get_flags(&self) -> u32;
    fn check_consistency(&self, ruleset: &RulesetCreated) -> Result<(), Error>;
}

#[derive(Debug, PartialEq)]
pub enum RulesetStatus {
    /// All requested restrictions are enforced.
    FullyEnforced,
    /// Some requested restrictions are enforced, following a best-effort approach.
    PartiallyEnforced,
    /// The running system doesn't support Landlock or a subset of the requested Landlock features.
    NotEnforced,
}

impl From<CompatState> for RulesetStatus {
    fn from(state: CompatState) -> Self {
        match state {
            CompatState::Start | CompatState::No | CompatState::Final => RulesetStatus::NotEnforced,
            CompatState::Full => RulesetStatus::FullyEnforced,
            CompatState::Partial => RulesetStatus::PartiallyEnforced,
        }
    }
}

/// Returned by ruleset builder.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct RestrictionStatus {
    /// Status of the Landlock ruleset enforcement.
    pub ruleset: RulesetStatus,
    /// Status of prctl(2)'s PR_SET_NO_NEW_PRIVS enforcement.
    pub no_new_privs: bool,
}

fn prctl_set_no_new_privs() -> Result<(), Error> {
    match unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } {
        0 => Ok(()),
        _ => Err(Error::last_os_error()),
    }
}

fn support_no_new_privs() -> bool {
    match unsafe { libc::prctl(libc::PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0) } {
        0 | 1 => true,
        // Only Linux < 3.5 or kernel with seccomp filters should return an error.
        _ => false,
    }
}

#[derive(Debug)]
pub struct RulesetInit {
    requested_handled_fs: BitFlags<AccessFs>,
    actual_handled_fs: BitFlags<AccessFs>,
    compat: Compatibility,
}

impl RulesetInit {
    pub fn new(compat: Compatibility) -> Self {
        // The API should be future-proof: one Rust program or library should have the same
        // behavior if built with an old or a newer crate (e.g. with an extended ruleset_attr
        // enum).  It should then not be possible to give an "all-possible-handled-accesses" to the
        // Ruleset builder because this value would be relative to the running kernel.
        //
        // TODO: Replace ABI::V1.into() with a Default implementation for BitFlags<_>
        let handled_fs = ABI::V1.into();
        RulesetInit {
            requested_handled_fs: handled_fs,
            actual_handled_fs: handled_fs,
            compat: compat,
        }
    }

    pub fn handle_fs<T>(mut self, access: T) -> Result<Self, Error>
    where
        T: Into<BitFlags<AccessFs>>,
    {
        self.requested_handled_fs = access.into();
        self.actual_handled_fs = self.requested_handled_fs.try_compat(&mut self.compat)?;
        Ok(self)
    }

    pub fn create(self) -> Result<RulesetCreated, Error> {
        let attr = uapi::landlock_ruleset_attr {
            handled_access_fs: self.actual_handled_fs.bits(),
        };

        match self.compat.abi {
            ABI::Unsupported => Ok(RulesetCreated {
                fd: -1,
                no_new_privs: true,
                requested_handled_fs: self.requested_handled_fs,
                compat: self.compat,
            }),
            ABI::V1 => match unsafe { uapi::landlock_create_ruleset(&attr, size_of_val(&attr), 0) }
            {
                fd if fd >= 0 => Ok(RulesetCreated {
                    fd: fd,
                    no_new_privs: true,
                    requested_handled_fs: self.requested_handled_fs,
                    compat: self.compat,
                }),
                _ => Err(Error::last_os_error()),
            },
        }
    }
}

#[derive(Debug)]
pub struct RulesetCreated {
    fd: RawFd,
    no_new_privs: bool,
    pub(crate) requested_handled_fs: BitFlags<AccessFs>,
    compat: Compatibility,
}

impl RulesetCreated {
    pub fn add_rule<T>(mut self, rule: T) -> Result<Self, Error>
    where
        T: Rule,
    {
        rule.check_consistency(&self)?;
        let compat_rule = rule.try_compat(&mut self.compat)?;
        match self.compat.abi {
            ABI::Unsupported => Ok(self),
            ABI::V1 => match unsafe {
                uapi::landlock_add_rule(
                    self.fd,
                    compat_rule.get_type_id(),
                    compat_rule.as_ptr(),
                    compat_rule.get_flags(),
                )
            } {
                0 => Ok(self),
                _ => Err(Error::last_os_error()),
            },
        }
    }

    pub fn set_no_new_privs(mut self, no_new_privs: bool) -> Self {
        self.no_new_privs = no_new_privs;
        self
    }

    pub fn restrict_self(mut self) -> Result<RestrictionStatus, Error> {
        let enforced_nnp = if self.no_new_privs {
            if let Err(e) = prctl_set_no_new_privs() {
                // To get a consistent behavior, calls this prctl whether or not Landlock is
                // supported by the running kernel.
                let support_nnp = support_no_new_privs();
                match self.compat.abi {
                    // It should not be an error for kernel (older than 3.5) not supporting
                    // no_new_privs.
                    ABI::Unsupported => {
                        if support_nnp {
                            // The kernel seems to be between 3.5 (included) and 5.13 (excluded),
                            // or Landlock is not enabled; no_new_privs should be supported anyway.
                            return Err(e);
                        }
                    }
                    // A kernel supporting Landlock should also support no_new_privs (unless
                    // filtered by seccomp).
                    _ => return Err(e),
                }
                false
            } else {
                true
            }
        } else {
            false
        };

        match self.compat.abi {
            ABI::Unsupported => Ok(RestrictionStatus {
                ruleset: self.compat.state.into(),
                no_new_privs: enforced_nnp,
            }),
            ABI::V1 => match unsafe { uapi::landlock_restrict_self(self.fd, 0) } {
                0 => {
                    self.compat.state.update(CompatState::Full);
                    Ok(RestrictionStatus {
                        ruleset: self.compat.state.into(),
                        no_new_privs: enforced_nnp,
                    })
                }
                _ => Err(Error::last_os_error()),
            },
        }
    }
}

impl Drop for RulesetCreated {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe { close(self.fd) };
        }
    }
}

#[test]
fn ruleset_unsupported() {
    use std::fs::File;
    use std::io::ErrorKind;

    let mut compat = Compatibility {
        abi: ABI::Unsupported,
        level: SupportLevel::Optional,
        state: CompatState::Start,
    };
    assert_eq!(
        RulesetInit::new(compat)
            .create()
            .unwrap()
            .restrict_self()
            .unwrap(),
        RestrictionStatus {
            ruleset: RulesetStatus::NotEnforced,
            no_new_privs: true,
        }
    );
    assert_eq!(
        RulesetInit::new(compat)
            .handle_fs(AccessFs::Execute)
            .unwrap()
            .create()
            .unwrap()
            .restrict_self()
            .unwrap(),
        RestrictionStatus {
            ruleset: RulesetStatus::NotEnforced,
            no_new_privs: true,
        }
    );

    assert_eq!(
        RulesetInit::new(compat)
            .create()
            .unwrap()
            .set_no_new_privs(false)
            .restrict_self()
            .unwrap(),
        RestrictionStatus {
            ruleset: RulesetStatus::NotEnforced,
            no_new_privs: false,
        }
    );

    assert_eq!(
        RulesetInit::new(compat)
            // Empty access-rights
            .handle_fs(ABI::Unsupported)
            .unwrap_err()
            .kind(),
        ErrorKind::Other
    );

    compat.abi = ABI::V1;
    assert_eq!(
        RulesetInit::new(compat)
            .handle_fs(AccessFs::Execute)
            .unwrap()
            .create()
            .unwrap()
            .restrict_self()
            .unwrap(),
        RestrictionStatus {
            ruleset: RulesetStatus::FullyEnforced,
            no_new_privs: true,
        }
    );
    assert_eq!(
        RulesetInit::new(compat)
            // Empty access-rights
            .handle_fs(ABI::Unsupported)
            .unwrap_err()
            .kind(),
        ErrorKind::Other
    );

    // Tests inconsistency between the ruleset handled access-rights and the rule access-rights.
    for handled_access in &[
        make_bitflags!(AccessFs::{Execute | WriteFile}),
        AccessFs::Execute.into(),
    ] {
        assert_eq!(
            RulesetInit::new(compat)
                .handle_fs(*handled_access)
                .unwrap()
                .create()
                .unwrap()
                .add_rule(
                    PathBeneath::new(&File::open("/").unwrap()).allow_access(AccessFs::ReadFile)
                )
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidInput
        );
    }
}
