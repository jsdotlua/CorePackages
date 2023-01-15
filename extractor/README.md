<!-- Allow this file to not have a first line heading -->
<!-- markdownlint-disable-file MD041 no-emphasis-as-heading -->

<!-- inline html -->
<!-- markdownlint-disable-file MD033 -->

<div align="center">

# `✨ CorePackages Extractor`

**Automatically extracts CorePackages from Roblox's CDN and processes them into a consumable format.**

</div>

## About

This program is designed to run autonomously every Roblox release. It works in multiple stages:

1. Download and extract the latest LuaPackages from Roblox's CDN. Currently, this only downloads the `LIVE` channel.

2. Process package source files and build a dependency graph for each package. This process uses the Rotriever `lock.toml` file to resolve dependencies.

3. Process source files and compute every license used by each package. This is a best-guess process, and if a license cannot be resolved for a source file with absolute certainty (currently > 95%) then the package will not be included in this repository or uploaded to any distribution platform.

4. TODO: Document the rest of the process

## About Package Licenses

To maintain legality, it is essential that the extractor correctly parses package licenses. As mentioned above, the extractor will attempt to match source license headers to a known license. If this cannot be done with above 95% certainty then the package *will not* be included in this public repository.

The dataset of known license headers that are matched against is located at [`datasets/license_headers.json`](datasets/license_headers.json).

If a package has dependencies (direct or transient) that are not appropriately licensed, then that package will also not be included. The extractor will automatically output information about packages that *are* appropriately licensed, but are blocked by a dependency from being included.

---

### Contribution 

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.1-ff69b4.svg)](CODE_OF_CONDUCT.md) 

I welcome contributions to this project! 

Please keep in mind that all code contributions should be made to this extractor because changes made directly to Lua packages will be overwritten!

### Extractor License 

This contribution is dual licensed under EITHER OF 

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>) 

- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>) 

at your option. 

For clarity, "your" refers to Brooke Rhodes or any other licensee/user of the 
contribution. 
