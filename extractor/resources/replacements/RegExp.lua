--[[
	* Copyright (c) Brooke Rhodes. All rights reserved.
	* Licensed under the MIT License (the "License");
	* you may not use this file except in compliance with the License.
	* You may obtain a copy of the License at
	*
	*     https://opensource.org/licenses/MIT
	*
	* Unless required by applicable law or agreed to in writing, software
	* distributed under the License is distributed on an "AS IS" BASIS,
	* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	* See the License for the specific language governing permissions and
	* limitations under the License.
]]

local RegEx = require(script.RegEx)

type RegExpExecArray = { string } & {
	index: number?,
	input: string?,
	n: number,
}

export type RegExp = {
	exec: (self: RegExp, input: string) -> RegExpExecArray | nil,
	test: (self: RegExp, input: string) -> boolean,
}

local RegExp = {}
RegEx.__index = RegExp

function RegExp:exec(str: string): RegExpExecArray | nil
	local match = self.innerRegEx:match(str)
	if match == nil then
		return nil
	end

	local index = match:span()
	local groups = match:grouparr()

	local matches = { groups[0] }

	for i = 1, groups.n do
		matches[i + 1] = groups[i]
	end

	matches.n = groups.n + 1
	matches.index = index
	matches.input = str

	return matches
end

function RegExp:test(str: string): boolean
	return self:exec(str) ~= nil
end

local function new(_self, pattern: RegExp | string, flags_: string?)
	local flags = flags_ or ""

	local innerRegEx = RegEx.new(pattern, flags)
	local self = setmetatable({
		source = pattern,
		ignoreCase = flags:find("i") ~= nil,
		global = flags:find("g") ~= nil,
		multiline = flags:find("m") ~= nil,
		innerRegEx = innerRegEx,
	}, RegEx)

	return self
end

local interface = setmetatable(RegExp, {
	__call = new,
})

return interface
