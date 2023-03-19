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
| `Boolean` | [`core-packages/boolean@1.2.2`](https://wally.run/package/core-packages/boolean) | N/A | N/A | N/A |
| `Collections` | [`core-packages/collections@1.2.2`](https://wally.run/package/core-packages/collections) | N/A | N/A | N/A |
| `Console` | [`core-packages/console@1.2.2`](https://wally.run/package/core-packages/console) | N/A | N/A | N/A |
| `DiffSequences-edcba0e9-2.4.1` | [`core-packages/diff-sequences@2.4.1`](https://wally.run/package/core-packages/diff-sequences) | N/A | N/A | N/A |
| `DiffSequences-edcba0e9-3.2.1` | [`core-packages/diff-sequences@3.2.1`](https://wally.run/package/core-packages/diff-sequences) | N/A | N/A | N/A |
| `ES7Types` | [`core-packages/es-7-types@1.2.2`](https://wally.run/package/core-packages/es-7-types) | N/A | N/A | N/A |
| `Emittery` | [`core-packages/emittery@3.2.1`](https://wally.run/package/core-packages/emittery) | N/A | N/A | N/A |
| `InstanceOf` | [`core-packages/instance-of@1.2.2`](https://wally.run/package/core-packages/instance-of) | N/A | N/A | N/A |
| `JestEnvironment` | [`core-packages/jest-environment@3.2.1`](https://wally.run/package/core-packages/jest-environment) | N/A | N/A | N/A |
| `JestEnvironmentLuau` | [`core-packages/jest-environment-luau@3.2.1`](https://wally.run/package/core-packages/jest-environment-luau) | N/A | N/A | N/A |
| `JestFakeTimers-edcba0e9-3.2.1` | [`core-packages/jest-fake-timers@3.2.1`](https://wally.run/package/core-packages/jest-fake-timers) | N/A | N/A | N/A |
| `JestGetType-edcba0e9-2.4.1` | [`core-packages/jest-get-type@2.4.1`](https://wally.run/package/core-packages/jest-get-type) | N/A | N/A | N/A |
| `JestGetType-edcba0e9-3.2.1` | [`core-packages/jest-get-type@3.2.1`](https://wally.run/package/core-packages/jest-get-type) | N/A | N/A | N/A |
| `JestMessageUtil-edcba0e9-2.4.1` | [`core-packages/jest-message-util@2.4.1`](https://wally.run/package/core-packages/jest-message-util) | N/A | N/A | N/A |
| `JestMock-edcba0e9-3.2.1` | [`core-packages/jest-mock@3.2.1`](https://wally.run/package/core-packages/jest-mock) | N/A | N/A | N/A |
| `JestTestResult-edcba0e9-2.4.1` | [`core-packages/jest-test-result@2.4.1`](https://wally.run/package/core-packages/jest-test-result) | N/A | N/A | N/A |
| `JestTestResult-edcba0e9-3.2.1` | [`core-packages/jest-test-result@3.2.1`](https://wally.run/package/core-packages/jest-test-result) | N/A | N/A | N/A |
| `JestTypes-edcba0e9-2.4.1` | [`core-packages/jest-types@2.4.1`](https://wally.run/package/core-packages/jest-types) | N/A | N/A | N/A |
| `JestTypes-edcba0e9-3.2.1` | [`core-packages/jest-types@3.2.1`](https://wally.run/package/core-packages/jest-types) | N/A | N/A | N/A |
| `JestValidate` | [`core-packages/jest-validate@3.2.1`](https://wally.run/package/core-packages/jest-validate) | N/A | N/A | N/A |
| `LuauPolyfill-2fca3173-1.2.2` | [`core-packages/luau-polyfill@1.2.2`](https://wally.run/package/core-packages/luau-polyfill) | N/A | N/A | N/A |
| `Math` | [`core-packages/math@1.2.2`](https://wally.run/package/core-packages/math) | N/A | N/A | N/A |
| `Number` | [`core-packages/number@1.2.2`](https://wally.run/package/core-packages/number) | N/A | N/A | N/A |
| `RegExp` | [`core-packages/reg-exp@0.2.0`](https://wally.run/package/core-packages/reg-exp) | N/A | N/A | N/A |
| `RobloxShared-edcba0e9-3.2.1` | [`core-packages/roblox-shared@3.2.1`](https://wally.run/package/core-packages/roblox-shared) | N/A | N/A | N/A |
| `Scheduler-07417f27-17.0.1-rc.17` | [`core-packages/scheduler@17.0.1-rc.17`](https://wally.run/package/core-packages/scheduler) | N/A | N/A | N/A |
| `Shared-07417f27-17.0.1-rc.17` | [`core-packages/shared@17.0.1-rc.17`](https://wally.run/package/core-packages/shared) | N/A | N/A | N/A |
| `Shared-9c8468d8-8a7220fd` | [`core-packages/shared@17.0.1-rc.16`](https://wally.run/package/core-packages/shared) | N/A | N/A | N/A |
| `Shared-a406e214-4230f473` | [`core-packages/shared@17.0.1-rc.18`](https://wally.run/package/core-packages/shared) | N/A | N/A | N/A |
| `String` | [`core-packages/string@1.2.2`](https://wally.run/package/core-packages/string) | N/A | N/A | N/A |
| `Symbol` | [`core-packages/symbol@1.0.0`](https://wally.run/package/core-packages/symbol) | N/A | N/A | N/A |
| `Throat` | [`core-packages/throat@3.2.1`](https://wally.run/package/core-packages/throat) | N/A | N/A | N/A |
| `Timers` | [`core-packages/timers@1.2.2`](https://wally.run/package/core-packages/timers) | N/A | N/A | N/A |
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

