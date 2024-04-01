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
Rido can be added to your project using 
`cargo add rido`

Optionally, you can enable enterprise windows releases with the `enterprise` feature.

A release can be fetched using the `new` method, which may return an error that must be handled.
For example:

```rust
use rido::WindowsRelease;
let release = WindowsRelease::new("10", "English (United States)")?;
```

The ConsumerRelease struct contains URL (String) and Hash (Option<String>) instance fields, which 
are populated by the `new` method.

## Available Releases and Languages

Windows 10 and 11: Arabic, Brazilian Portuguese, Bulgarian, Chinese (Simplified), Chinese (Traditional), Croatian, Czech, Danish, Dutch, English (United States), English International, Estonian, Finnish, French, French Canadian, German, Greek, Hebrew, Hungarian, Italian, Japanese, Korean, Latvian, Lithuanian, Norwegian, Polish, Portuguese, Romanian, Russian, Serbian Latin, Slovak, Slovenian, Spanish, Spanish (Mexico), Swedish, Thai, Turkish, Ukrainian

Windows 8: Arabic, Brazilian Portuguese, Bulgarian, Chinese (Simplified), Chinese (Traditional), Chinese (Traditional Hong Kong), Croatian, Czech, Danish, Dutch, English (United States), English International, Estonian, Finnish, French, German, Greek, Hebrew, Hungarian, Italian, Japanese, Latvian, Lithuanian, Norwegian, Polish, Portuguese, Romanian, Russian, Serbian Latin, Slovak, Slovenian, Spanish, Swedish, Thai, Turkish, Ukrainian

Enterprise: 

Windows 10-ltsc, 10-enterprise, 11-enterprise: Various languages (to be added to documentation later).
