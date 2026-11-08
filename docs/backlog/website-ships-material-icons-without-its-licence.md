# The website ships Material Icons without the Apache License text

Observed: the website that `nectar build` writes carries the theme's fonts and the licence texts
of Ubuntu Sans, Inter, IBM Plex Mono, Victor Mono and Paged.js, which `crates/html/src/lib.rs`
embeds from `theme/static/`. Material Icons ships with the site as well, but its licence text,
the Apache License 2.0, lives in `crates/html/theme/LICENSE-Apache-2.0.txt`, outside `static/`,
and no output carries it. Found in the review of the release archive's notices (ticket
revision-2026-11-07/06), whose scope was the archive.

Where: crates/html/src/lib.rs (the embedded static files), crates/html/theme/NOTICE,
crates/html/theme/LICENSE-Apache-2.0.txt.
