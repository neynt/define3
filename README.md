![](https://i.neynt.ca/Gsb1uv1VykfNaYRe.png)

---

Offline command-line dictionary based on Wiktionary, thus supporting basically
every word ever.

![](https://i.neynt.ca/k1UhBbvpKqJ57q1z.png)

## Usage

- Download the Wiktionary `pages-articles.xml.bz2` dump.
  - [https://dumps.wikimedia.org/enwiktionary/](https://dumps.wikimedia.org/enwiktionary/)
- Extract the bz2.
- Run `build_definitions_db` on the resulting xml file.
- Run `define` to define words.

## TODO

- Detect languages and parts of speech automatically (is currently hardcoded)
- Render Wiki templates
  - This is super hard because many templates eventually resolve to Scribunto
    scripts, so we have to call into Lua
- Resolve inflections
  - Probably has to be language-specific
- Additional data sets
  - The ones Tangorin uses seem quite nice for Japanese
