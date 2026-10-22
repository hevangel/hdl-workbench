# Vendored simulator sources

`xezim/` is Xezim revision `22faa127a9cc0156bd23e0ab117f400c64231913`.
`xezim-core/` (including `xezim-parser/`) is revision
`d15cdc39e27a4c0a8a55f7e1d46369181307ead9`.

Both upstream projects are licensed `MIT OR Apache-2.0`; their license files
are kept in the corresponding source trees. The local copies were created from
`git archive` at those commits, without `.git` metadata. LUNA-03 portability
changes are applied directly to the vendored Xezim manifest, build script,
and library entry points. Native behavior remains the upstream default.
