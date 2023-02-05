<!-- Allow this file to not have a first line heading -->
<!-- markdownlint-disable-file MD041 no-emphasis-as-heading -->

<!-- inline html -->
<!-- markdownlint-disable-file MD033 -->

<div align="center">

# `📁 CorePackages`

**A collection of Roblox's licensed CorePackages, automatically prepared for [Wally](https://wally.run).**

</div>

### About

CorePackages (also known as LuaPackages) are Roblox-internal code packages, most of which are not useful to game developers. However, over the past couple of years, Roblox has begun translating a handful of JavaScript packages to Luau. Examples of these packages include [React](https://reactjs.org/), [Jest](https://jestjs.io/), [GraphQL](https://graphql.org/), and [Apollo](https://www.apollographql.com/). Many of these translated packages are of great use to game and library developers on Roblox. Even better, the majority are under an appropriate open-source license!

Despite the great potential use to many developers, these translated packages are not published by Roblox and are only available inside of Roblox client releases. The purpose of this project is to automate the extraction process of all licensed CorePackages and to archive them in this repository (and on [Wally](https://wally.run/)!)

This project comes in two parts:

1. An [`extractor/`](/extractor/) for extracting any CorePackage and its dependencies from Roblox's CorePackages. The latest packages are automatically pulled from Roblox's CDN.
2. The [`modules/`](/modules/) available by default on Wally (under the `core-packages` scope). Everything in this repository is properly licensed (see below) under appropriate open-source licenses that enable use in your projects.

The extraction process is tested with comprehensive unit and integration tests. In addition, some CorePackages come with their own test suites that are executed every extractor run to ensure correctness. Code available in Roblox's CorePackages is already deployed to millions of users and hundreds of developers worldwide, and as such, stability may as well be a guarantee*.

### Documentation

One potential challenge with using Roblox's CorePackages is a lack of documentation. It's important to remember that, if you are using a translated package (such as React or Jest), you can just use their official documentation! Despite being translated, you *are* using the real-deal. Their use is near-enough 1:1 with the upstream JavaScript package.

However, there are some cases that are Roblox-specific. One such case is migration from [*Roact*](https://github.com/Roblox/roact) to *React* (often referred to as Roact17 by Roblox). Unfortunately, cases like these are a matter of the community documenting pitfalls they come across. Community documentation for various packages in this repository is available on our [docs website](#). If you come across hurdles and errors you had to resolve when using a CorePackage, please help others out by contributing to our documentation (see our [contribution guide](/CONTRIBUTION.md))!

### Package References

#### Available Packages

Below is a reference to all CorePackages currently available on Wally under the `core-packages` scope. Some packages you may be interested in could be blocked by unlicensed code, so keep reading for references on different unlicensed packages.

| Original Name | Wally Package | License(s) | Upstream Repository | Types Repository |
| ------------- | ------------- | ---------- | ------------------- | ---------------- |
| `Jest-edcba0e9-3.2.1` | [`core-packages/jest@3.2.1`](https://wally.run/package/core-packages/jest) | N/A | N/A | N/A |
| `TestUtils-edcba0e9-3.2.1` | [`core-packages/test-utils@3.2.1`](https://wally.run/package/core-packages/test-utils) | N/A | N/A | N/A |
| `ReactReconciler-a406e214-4230f473` | [`core-packages/react-reconciler@17.0.1-rc.18`](https://wally.run/package/core-packages/react-reconciler) | N/A | N/A | N/A |
| `ReactReconciler-9c8468d8-8a7220fd` | [`core-packages/react-reconciler@17.0.1-rc.16`](https://wally.run/package/core-packages/react-reconciler) | N/A | N/A | N/A |
| `JestConfig` | [`core-packages/jest-config@3.2.1`](https://wally.run/package/core-packages/jest-config) | N/A | N/A | N/A |
| `JestCore` | [`core-packages/jest-core@3.2.1`](https://wally.run/package/core-packages/jest-core) | N/A | N/A | N/A |
| `ES7Types` | [`core-packages/es-7-types@1.1.0`](https://wally.run/package/core-packages/es-7-types) | N/A | N/A | N/A |
| `JestSnapshot-edcba0e9-3.2.1` | [`core-packages/jest-snapshot@3.2.1`](https://wally.run/package/core-packages/jest-snapshot) | N/A | N/A | N/A |
| `JestReporters-edcba0e9-2.4.1` | [`core-packages/jest-reporters@2.4.1`](https://wally.run/package/core-packages/jest-reporters) | N/A | N/A | N/A |
| `JestRuntime` | [`core-packages/jest-runtime@3.2.1`](https://wally.run/package/core-packages/jest-runtime) | N/A | N/A | N/A |
| `JestEnvironmentLuau` | [`core-packages/jest-environment-luau@3.2.1`](https://wally.run/package/core-packages/jest-environment-luau) | N/A | N/A | N/A |
| `JestReporters-edcba0e9-3.2.1` | [`core-packages/jest-reporters@3.2.1`](https://wally.run/package/core-packages/jest-reporters) | N/A | N/A | N/A |
#### Blocked Packages

While a package may be licensed, it could be blocked from being included by dependencies (direct or transient). Below is a reference to all packages blocked from being included by one or more unlicensed dependencies.

| Original Name | Version | Blocking Dependencies | Upstream Repository | Types Repository |
| ------------- | ------- | --------------------- | ------------------- | ---------------- |
#### Blocking Packages

Below is a reference to all packages that are blocking other packages from being included. This reference is mostly useful for prioritizing what should be rewritten under an open-source license, so that other packages may be included.

| Original Name | Version | Blocked Count | Blocked Packages | Upstream Repository | Types Repository |
| ------------- | ------- | ------------- | ---------------- | ------------------- | ---------------- |
#### Unlicensed Packages

Below is a reference to all packages that are simply unlicensed. Most are of no use to developers.

<details>
<summary>Click to expand</summary>

| Original Name | Version |
| ------------- | ------- |
</details>

