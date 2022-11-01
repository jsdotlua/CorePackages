local ReplicatedStorage = game:GetService("ReplicatedStorage")
local Players = game:GetService("Players")

local Packages = ReplicatedStorage:WaitForChild("Packages")
local Roact = require(Packages:WaitForChild("Roact"))

local e = Roact.createElement
local useState = Roact.useState

local function Button()
    local clicks, setClicks = useState(0)
    
    return e("TextButton", {
        AnchorPoint = Vector2.new(0.5, 0.5),
        Position = UDim2.fromScale(0.5, 0.5),
        Size = UDim2.fromOffset(240, 240),

        Text = "Clicks: " .. clicks,

        [Roact.Event.Activated] = function()
            setClicks(clicks + 1)
        end,
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
