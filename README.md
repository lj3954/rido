## Rido
Rido is a library crate which allows applications access to
valid URLs and Checksums for various releases of Microsoft Windows. 
It is inspired by the [Mido](https://github.com/ElliotKillick/Mido) bash script and
[Fido](https://github.com/pbatard/Fido) PowerShell script. 

## License
Rido is licensed under the [GNU General Public License Version 3](https://www.gnu.org/licenses/gpl-3.0). This license prohibits the use of this crate as a library within 
nonfree software or any software licensed under incompatible licenses
which enable the production of nonfree software, such as the MIT license.

## Usage
Rido can be added to your project using 
`cargo add rido`

Optionally, you can enable enterprise windows releases with the `enterprise` feature.

A release can be fetched using the `new` method, which may return an error that must be handled.
For example:

```rust
use rido::WindowsData;
let release = WindowsData::new("10", "English (United States)", "x86_64")?;
```

Optionally, for a specific Product ID can be specified for Consumer windows releases.
This allows for pinning to a specific release, so long as it remains publicly available.
This can be done by replacing the release string with ```"productid:id"```. 

The WindowsData struct contains URL (`String`) and Hash (`Option<String>`) instance fields, which 
are populated by the `new` method.

Rido also supports downloading both 32-bit and 64-bit images for operating systems that support them. Windows 10 (including enterprise) releases offer 32-bit images. Use the i686 architecture to specify a 32-bit image, and x86_64 for a 64-bit image.

Rido includes an Architecture enum and release/language enums for each of consumer & enterprise. Alternatively, as in the example above, you may use ```&str```s, since ```TryInto<&str>``` is implemented for each and the new function will take in any type implementing TryInto;

You can also build a WindowsEntry with release, language, and architecture fields, or gather a vector of all available entries with the "list_all" method. WindowsData implements ```TryFrom<WindowsEntry>```

## Available Releases and Languages

10/11: Arabic, Brazilian Portuguese, Bulgarian, Chinese (Simplified), Chinese (Traditional), Croatian, Czech, Danish, Dutch, English (United States), English International, Estonian, Finnish, French, French Canadian, German, Greek, Hebrew, Hungarian, Italian, Japanese, Korean, Latvian, Lithuanian, Norwegian, Polish, Portuguese, Romanian, Russian, Serbian Latin, Slovak, Slovenian, Spanish, Spanish (Mexico), Swedish, Thai, Turkish, Ukrainian

Enterprise: 

10-ltsc/10-enterprise/11-enterprise: English (United States), English (Great Britain), Chinese (Simplified), Chinese (Traditional), French, German, Italian, Japanese, Korean, Portuguese (Brazil), Spanish

server-2012-r2/server-2016/server-2019/server-2022: English (United States), Chinese (Simplified), French, German, Italian, Japanese, Russian, Spanish
