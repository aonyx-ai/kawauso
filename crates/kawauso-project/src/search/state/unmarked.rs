//! The state of a search that names no marker

/// The state of a search that names no marker
///
/// A search in this state cannot discover a project, because a walk without a
/// marker tests nothing and reaches the root of the file system. The type
/// exists so that the compiler reports this, and not the first run of the
/// application.
///
/// A constructor of a search returns this state, and the first marker leaves
/// it. The type has no value that carries information; it only names the
/// state of a search.
///
/// # Examples
///
/// ```
/// use kawauso_project::Search;
///
/// use kawauso_project::search::state::Unmarked;
///
/// let search: Search<Unmarked> = Search::start(".");
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[non_exhaustive]
pub struct Unmarked;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmarked_is_send_sync_and_unpin() {
        fn assert_send_sync_and_unpin<T: Send + Sync + Unpin>() {}

        assert_send_sync_and_unpin::<Unmarked>();
    }
}
