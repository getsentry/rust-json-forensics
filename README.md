# rust-json-forensics

This crate exposes a function that takes a byteslice and:

* Converts the invalid JSON tokens `NaN` and `Infinity` into `0`
* Replaces all integers that would cause an overflow in `serde-json` with `0`

This is just to get the JSON to parse. All operations happen in-place.

This is useful because the Python JSON library traditionally emits invalid
JSON if `NaN` and `Infinity` values are encountered.  If you have to support
clients like this, this wrapper can be used to still deserialize such a
JSON document.


Successor of [python-json-read-adapter](https://github.com/mitsuhiko/python-json-read-adapter)
