use libspa_macro::enum_from_sys;

enum_from_sys!(MediaType, r"SPA_MEDIA_TYPE_([a-z0-9]+)");

enum_from_sys!(MediaSubtype, r"SPA_MEDIA_SUBTYPE_([a-z0-9]+)");
