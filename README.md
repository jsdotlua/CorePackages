# Roact17

Roact17 is Roblox's internal port of React, transpiled directly from React's source code. 

Unlike [Roact](https://github.com/Roblox/roact), which is based on React but has unique implementation details, Roact17 *is* React. Excluding some Luau polyfills and a custom Roblox renderer, Roact17 is identical to React internally and follows the same internal semantics. The advantage here is that React engineers can follow documentation almost 1:1, making hiring for teams much more accessible.

Roblox uses Roact17 internally for their universal Lua app, which runs on mobile and desktop. Builtin plugins are also being migrated to Roact17. Roblox's motivation for doing so is likely (*purely speculation*) that it makes onboarding for app developers easier because they can hire people with a strong React background.

Should you use Roact17? Probably not.

TODO: Write rest of README
