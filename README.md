![](https://i.neynt.ca/Gsb1uv1VykfNaYRe.png)

---

Offline command-line dictionary based on Wiktionary, thus supporting every word
in every language.

![](https://i.neynt.ca/k1UhBbvpKqJ57q1z.png)

## TODO

- Make it possible for other people to use by not hardcoding a billion things.
- Detect languages and parts of speech automatically (is currently hardcoded)
- Render Wiki templates
  - This is super hard because many templates eventually resolve to Scribunto
    scripts, so we have to call into Lua
- Resolve inflections
  - Probably has to be language-specific
- Additional data sets
  - The ones Tangorin uses seem quite nice for Japanese
