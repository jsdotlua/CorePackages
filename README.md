<!-- Allow this file to not have a first line heading -->
<!-- markdownlint-disable-file MD041 no-emphasis-as-heading -->

<!-- inline html -->
<!-- markdownlint-disable-file MD033 -->

<div align="center">

# `📁 CorePackages`

**A collection of Roblox's MIT and Apache2-licensed CorePackages, automatically prepared for [Wally](https://wally.run).**

</div>

**NOTE:** This repository is no longer directly maintained! All packages families (React, Jest, etc.) now live in their own repositories. Check our [org page](https://github.com/jsdotlua) for more.

## About

This project comes in two parts:
1. An [`extractor/`](/extractor/) for extracting any CorePackage and its dependencies from Roblox's CorePackages.
2. The [`modules/`](/modules/) available by default on Wally (under the `core-packages` scope). Everything in this repository is properly licensed (see below) under appropriate open-source licenses that enable use in your projects.

The extractor in its current state is pretty cobbled together, with a few dependency resolution bugs. I plan to improve its implementation when I find the time.

## Motivation

Roblox has fantastic internal packages, and they're going MIT (or Apache2)!

Roblox's Lua teams have been pushing internally to make their hard work available to the community. This move is excellent, but that work probably won't be available on places like GitHub for a while. So, I decided to make all the ready-to-use packages available here under the relevant open licenses.

But why should you *care* about Roblox's internal Packages?

## Roact17

My primary motivator for this project has been Roblox's Roact17, a direct transpilation of React (version 17). While big (over 45,000 lines of code), Roact17 boasts *significant* performance benefits over standard Roact, Fusion, and even traditional UI programming. This performance is thanks to many things, such as Roact17's concurrent renderer and the fact Roblox has been investing *lots* in Roact17's performance.

Roact17 is the technology powering Roblox's universal Lua app and (soon) Studio's built-in Lua plugins.

Roact17 is available on Wally [here](https://wally.run/package/core-packages/roact-compat).

## GraphQL and Apollo

While not yet available in this repository, Roblox's CorePackages also includes Facebook's GraphQL client and Apollo.

These are not included in this repository because:
- The GraphQL client still has about a dozen unlicensed files, which I need to find the time to rewrite.
- The Apollo client doesn't include any license headers, despite being transpiled. Hopefully, Roblox will include license headers soon.

## Contribution

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.1-ff69b4.svg)](CODE_OF_CONDUCT.md)

I welcome community contributions to this project. If you want to contribute, please make changes to the [`extractor/`](/extractor/) only. Any changes made directly to packages will be overwritten by the extractor.

## License
As a result of how Roblox has structured its internal modules, each script includes its license information in the header (usually licensed under either Roblox, Facebook, or me).
