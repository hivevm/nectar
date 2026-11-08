# Diagnostics

Problems are reported with file and line. `nectar build` fails on errors and lists warnings;
`nectar serve` also shows them in a panel on every page and keeps watching. Raw HTML shown as text
is reported, and the PDF and the EPUB report what they leave out: raw HTML where `html = true`,
and images they could not embed.
