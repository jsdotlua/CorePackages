local ReplicatedStorage = game:GetService("ReplicatedStorage")
local Players = game:GetService("Players")

local Packages = ReplicatedStorage:WaitForChild("Packages")
local Roact = require(Packages:WaitForChild("Roact"))

local e = Roact.createElement

local function Button()
    return e("TextButton", {
        AnchorPoint = Vector2.new(0.5, 0.5),
        Position = UDim2.fromScale(0.5, 0.5),
        Size = UDim2.fromOffset(240, 240),
    })
end

local function SetupUi()
    local player = Players.LocalPlayer

    local root = e("ScreenGui", {}, {
        e(Button),
    })

    Roact.mount(root, player.PlayerGui, "ExampleUI")
end

SetupUi()
