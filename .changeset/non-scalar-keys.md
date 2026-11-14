---
front-matter: patch
"verctl:group": fixed
---

A YAML key that is a list, a mapping or an alias, such as `? [a, b]`, no longer makes `read` fail.
Its entry gives no value, and the entries after it are read.
