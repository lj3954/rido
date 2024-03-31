## Rido
Rido is a library crate which allows applications access to
valid URLs and Checksums for various releases of Microsoft Windows. 
It is inspired by the [Mido](https://github.com/ElliotKillick/Mido) bash script and
[Fido](https://github.com/pbatard/Fido) PowerShell script. 

## License
Rido is licensed under the [GNU General Public License Version 3](https://www.gnu.org/licenses/gpl-3.0). This license prohibits the use of this library within 
nonfree software or any software licensed under incompatible licenses
which enable the production of nonfree software, such as the MIT license.

## Usage
(TBD)
Rido can be added to your project using 
`cargo add rido --features consumer`.

A Consumer release can be fetched using the `new` method, which may return an error that must be handled.
For example:

```rust
use rido::ConsumerRelease;
let release = ConsumerRelease::new("10", "English (United States)")?;
```

The ConsumerRelease struct contains URL and Hash instance fields, which 
are populated by the `new` method.

