<!-- Allow this file to not have a first line heading -->
<!-- markdownlint-disable-file MD041 no-emphasis-as-heading -->

<!-- inline html -->
<!-- markdownlint-disable-file MD033 -->

<div align="center">

# `📁 CorePackages`

**A collection of Roblox's MIT or Apache2-licensed CorePackages, automatically packaged for [Wally](https://wally.run).**

</div>

## About

This project comes in two parts:
1. An [`extractor/`](/extractor/) for extracting any CorePackage and its dependencies from Roblox's CorePackages.
2. The [`modules/`](/modules/) available by default on Wally (under the `core-packages` scope). Everything in this repository is properly licensed (see below) under appropriate open-source licenses that enable use in your own projects.

The extractor in its current state is pretty cobbled together, still with a few dependency resolution bugs. I hope to improve its implementation when I find time.

## Motivation

Roblox has awesome internal packages, and they're going MIT (or Apache2)!

Roblox's Lua teams have been pushing hard internally to make their hard work available to the community. This is great, but that work probably won't be available on places like GitHub any time soon. So, I decided to make all the ready-to-use packages available here, under the relevant open licenses.

But, why should you *care* about Roblox's internal Packages?

## Roact17

My primary motivator for this project has been Roblox's own Roact17, which is a direct transpilation of React (version 17). While big (over 45,000 lines of code), Roact17 boasts *significant* performance benefits over standard Roact, Fusion, and even traditional UI programming. This performance is thanks to many things, such as Roact17's concurrent renderer and the fact Roblox has been investing *lots* in Roact17's performance.

Roact17 is the technology powering Roblox's universal Lua app and (soon) Studio's built-in Lua plugins.

Roact17 is available on Wally [here](https://wally.run/package/core-packages/roactCompat).

## GraphQL and Apollo

While not yet available in this repository, Roblox's CorePackages also includes Facebook's GraphQL client and Apollo.

These are not included in this repository because:
- The GraphQL client still has about a dozen unlicensed files which I need to find the time to rewrite.
- The Apollo client doesn't include any license headers at all, despite being transpiled. Hopefully, license headers will be included soon.

## Contribution

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v2.1-ff69b4.svg)](CODE_OF_CONDUCT.md)

I welcome community contributions to this project. If you want to contribute, then please make your changes to the [`extractor/`](/extractor/) only. Any changes made directly to packages will be overwritten.

## License
As a result of how Roblox has structured their internal modules, each script includes its own license information in the header (usually licensed under either Roblox, Facebook, or me).

This repository contains top-level copies of all licenses in use.
