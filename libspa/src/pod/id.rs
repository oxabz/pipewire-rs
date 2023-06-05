use libspa_macro::enum_from_sys;

use crate::utils::Id;

pub trait IdEnum: Sized + Copy + Clone + From<Id> + Into<Id> + for<'a> From<&'a Id> {}

enum_from_sys!(MediaType, r"SPA_MEDIA_TYPE_([a-z0-9]+)");

enum_from_sys!(MediaSubtype, r"SPA_MEDIA_SUBTYPE_([a-z0-9]+)");

enum_from_sys!(Format, r"SPA_VIDEO_FORMAT_([A-Za-z0-9]+_[A-Za-z0-9]+)");

