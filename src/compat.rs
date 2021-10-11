use crate::{uapi, BitFlags};
use enumflags2::BitFlag;
use std::convert::{TryFrom, TryInto};
use std::io::{Error, ErrorKind};

#[cfg(test)]
use crate::{make_bitflags, AccessFs};

/// Version of the Landlock [ABI](https://en.wikipedia.org/wiki/Application_binary_interface).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ABI {
    Unsupported = 0,
    V1 = 1,
}

impl ABI {
    fn new_current() -> Result<Self, Error> {
        unsafe {
            // Landlock ABI version starts at 1 but errno is only set for negative values.
            uapi::landlock_create_ruleset(
                std::ptr::null(),
                0,
                uapi::LANDLOCK_CREATE_RULESET_VERSION,
            )
        }
        .try_into()
    }
}

impl TryFrom<i32> for ABI {
    type Error = Error;

    fn try_from(value: i32) -> Result<ABI, Error> {
        const EOPNOTSUPP: i32 = -libc::EOPNOTSUPP;
        const ENOSYS: i32 = -libc::ENOSYS;

        match value {
            EOPNOTSUPP => Ok(ABI::Unsupported),
            ENOSYS => Ok(ABI::Unsupported),
            // A value of 0 should not come from the kernel, but if it is the case we get an
            // Other error (or an Uncategorized error in a newer Rust std).
            n if n <= 0 => Err(Error::from_raw_os_error(n * -1)),
            1 => Ok(ABI::V1),
            // Returns the greatest known ABI.
            _ => Ok(ABI::V1),
        }
    }
}

#[test]
fn abi_try_from() {
    assert_eq!(
        ABI::try_from(-1).unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );
    assert_eq!(ABI::try_from(0).unwrap_err().kind(), ErrorKind::Other);

    // EOPNOTSUPP
    assert_eq!(ABI::try_from(-95).unwrap(), ABI::Unsupported);
    // ENOSYS
    assert_eq!(ABI::try_from(-38).unwrap(), ABI::Unsupported);

    assert_eq!(ABI::try_from(1).unwrap(), ABI::V1);
    assert_eq!(ABI::try_from(2).unwrap(), ABI::V1);
    assert_eq!(ABI::try_from(9).unwrap(), ABI::V1);
}

/// Returned by ruleset builder.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum CompatState {
    /// Initial unknown state.
    Start,
    /// All requested restrictions are enforced.
    Full,
    /// Some requested restrictions are enforced, following a best-effort approach.
    Partial,
    /// The running system doesn't support Landlock.
    No,
    /// Final unsupported state.
    Final,
}

impl CompatState {
    pub(crate) fn update(&mut self, other: Self) {
        *self = match (*self, other) {
            (CompatState::Final, _) => CompatState::Final,
            (_, CompatState::Final) => CompatState::Final,
            (CompatState::Start, state) => state,
            (state, CompatState::Start) => state,
            (CompatState::No, CompatState::No) => CompatState::No,
            (CompatState::Full, CompatState::Full) => CompatState::Full,
            (_, _) => CompatState::Partial,
        }
    }
}

#[test]
fn compat_state_update_1() {
    let mut state = CompatState::Start;

    state.update(CompatState::Start);
    assert_eq!(state, CompatState::Start);

    state.update(CompatState::No);
    assert_eq!(state, CompatState::No);

    state.update(CompatState::Start);
    assert_eq!(state, CompatState::No);

    state.update(CompatState::Full);
    assert_eq!(state, CompatState::Partial);

    state.update(CompatState::Start);
    assert_eq!(state, CompatState::Partial);

    state.update(CompatState::No);
    assert_eq!(state, CompatState::Partial);

    state.update(CompatState::Final);
    assert_eq!(state, CompatState::Final);

    state.update(CompatState::Full);
    assert_eq!(state, CompatState::Final);

    state.update(CompatState::Start);
    assert_eq!(state, CompatState::Final);
}

#[test]
fn compat_state_update_2() {
    let mut state = CompatState::Full;

    state.update(CompatState::Full);
    assert_eq!(state, CompatState::Full);

    state.update(CompatState::No);
    assert_eq!(state, CompatState::Partial);

    state.update(CompatState::Start);
    assert_eq!(state, CompatState::Partial);
}

#[derive(Copy, Clone, Debug)]
pub enum SupportLevel {
    /// Best-effort security approach, should be selected by default.
    Optional,
    /// Strict security requirement (e.g., to return an error if not all requested security
    /// features are supported).
    Required,
}

// FIXME: remove Copy, it is too easy to misuse a builder pattern:
// compat.set_support_level(SupportLevel::Required);
// then use (unmodified) compat somehow…
/// Properly handles runtime unsupported features.  This enables to guarantee consistent behaviors
/// across crate users and runtime kernels even if this crate get new features.  It eases backward
/// compatibility and enables future-proofness.
///
/// Landlock is a security feature designed to help improve security of a running system thanks to
/// application developers.  To protect users as much as possible, compatibility with the running
/// system should then be handled in a best-effort way, contrary to common system features.  In
/// some circumstances (e.g. applications carefully designed to only be run with a specific kernel
/// version), it may be required to check if some of there features are enforced, which is possible
/// with XXX
#[derive(Copy, Clone, Debug)]
pub struct Compatibility {
    pub(crate) abi: ABI,
    pub(crate) level: SupportLevel,
    pub(crate) state: CompatState,
}

impl Compatibility {
    pub fn new() -> Result<Compatibility, Error> {
        let abi = ABI::new_current()?;
        Ok(Compatibility {
            abi: abi,
            level: SupportLevel::Optional,
            state: match abi {
                // Forces the state as unsupported because all possible types will be useless.
                ABI::Unsupported => CompatState::Final,
                _ => CompatState::Start,
            },
        })
    }

    pub fn set_support_level(mut self, level: SupportLevel) -> Self {
        self.level = level;
        self
    }
}

pub trait TryCompat {
    fn try_compat(self, compat: &mut Compatibility) -> Result<Self, Error>
    where
        Self: Sized;
}

impl<T> TryCompat for BitFlags<T>
where
    T: BitFlag,
    BitFlags<T>: From<ABI>,
{
    fn try_compat(self, compat: &mut Compatibility) -> Result<Self, Error> {
        let access_mask = match compat.level {
            SupportLevel::Optional => Self::all(),
            SupportLevel::Required => Self::from(compat.abi),
        };
        let (state, ret) = if self.is_empty() {
            // Empty access-rights would result to a runtime error.
            (
                CompatState::Final,
                Err(Error::from_raw_os_error(libc::ENOMSG)),
            )
        } else if !access_mask.contains(self) {
            // Unknown access-rights would result to a runtime error.
            (
                CompatState::Final,
                Err(Error::from_raw_os_error(libc::ENOMSG)),
            )
        } else {
            let compat_bits = self & Self::from(compat.abi);
            if compat_bits.is_empty() {
                (
                    CompatState::No,
                    match compat.level {
                        SupportLevel::Optional => Ok(compat_bits),
                        SupportLevel::Required => {
                            Err(Error::new(ErrorKind::InvalidData, "Incompatibility"))
                        }
                    },
                )
            } else if compat_bits != self {
                (
                    CompatState::Partial,
                    match compat.level {
                        SupportLevel::Optional => Ok(compat_bits),
                        SupportLevel::Required => {
                            Err(Error::new(ErrorKind::InvalidData, "Partial compatibility"))
                        }
                    },
                )
            } else {
                (CompatState::Full, Ok(compat_bits))
            }
        };
        compat.state.update(state);
        ret
    }
}

#[test]
fn compat_bit_flags() {
    let mut compat = Compatibility {
        abi: ABI::V1,
        level: SupportLevel::Optional,
        state: CompatState::Start,
    };

    let ro_access = make_bitflags!(AccessFs::{Execute | ReadFile | ReadDir});
    assert_eq!(ro_access, ro_access.try_compat(&mut compat).unwrap());

    let empty_access = BitFlags::<AccessFs>::empty();
    assert_eq!(
        ErrorKind::Other,
        empty_access.try_compat(&mut compat).unwrap_err().kind()
    );

    let all_unknown_access = unsafe { BitFlags::<AccessFs>::from_bits_unchecked(1 << 63) };
    assert_eq!(
        ErrorKind::Other,
        all_unknown_access
            .try_compat(&mut compat)
            .unwrap_err()
            .kind()
    );

    let some_unknown_access = unsafe { BitFlags::<AccessFs>::from_bits_unchecked(1 << 63 | 1) };
    assert_eq!(
        ErrorKind::Other,
        some_unknown_access
            .try_compat(&mut compat)
            .unwrap_err()
            .kind()
    );

    // Access-rights are valid (but ignored) when they are not required for the current ABI.
    compat.abi = ABI::Unsupported;
    assert_eq!(empty_access, ro_access.try_compat(&mut compat).unwrap());

    // Access-rights are not valid when they are required for the current ABI.
    compat.level = SupportLevel::Required;
    assert_eq!(
        ErrorKind::Other,
        ro_access.try_compat(&mut compat).unwrap_err().kind()
    );
}