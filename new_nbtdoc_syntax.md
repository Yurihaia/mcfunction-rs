## Tokens

### Quoted String: `QUOTED_STRING`

> `"` ( `\` ~[ `\n` `\r` ] | ~[ `\n` `\r` `"` `\` ] )<sup>\*</sup> `"`

### Identifier: `IDENT`

> [ `a`-`z`, `A`-`Z`, `_` ]<sup>+</sup>

### Whitespace: `WHITESPACE`

> [ `\u0020` `\t` `\n` `\f` `\r` ]<sup>\*</sup>

`\u0020` is a space

### Comment: `COMMENT`

> `//` ~[ `\n` ]<sup>\*</sup>

### Doc Comment: `DOC_COMMENT`

> `///` ~[ `\n` ]<sup>\*</sup>

### Float: `FLOAT`

> `-`<sup>?</sup> ( \
> &nbsp;&nbsp; `.` [ `0`-`9` ]<sup>+</sup>\
> &nbsp;&nbsp; | [ `0`-`9` ]<sup>+</sup> ( `.` [ `0`-`9` ]<sup>+</sup> )<sup>?</sup>\
> )\
> ( [ `e` `E` ][ `-`, `+` ]<sup>?</sup> [ `0`-`9` ]<sup>+</sup> )<sup>?</sup>

### Integer: `INTEGER`

> `-`<sup>?</sup> [ `0`-`9` ]<sup>+</sup>

Parsing automatically skips `WHITESPACE`, `COMMENT`, and `DOC_COMMENT` tokens unless specified.

## Parser Rules
