-- This is a sample init.lua file that shows the options for the intenral api


--[[ Supported builtin themes are:
Pinel Blueberry Dark
Pinel Blueberry Light
Catppuccin Mocha
Gruvbox Dark
GitHub Dark
Nord
TokyoNight
Ayu Dark

You may refer to them as `pinel.theme.use_builtin(<Name>)` where `Name` is one of the options
above
]]
pinel.theme.use_builtin("TokyoNight")


--[[
Below are the options for the bg_* and text_* and border_* and misc attributes that might be helpful
The first parameter is the name of the visual section and the second parameter is the HEX value assigned to it.

pinel.theme.set_color("bg_primary", "#1a1f2b")
pinel.theme.set_color("bg_secondary", "#161b24")
pinel.theme.set_color("bg_editor", "#0f141c")
pinel.theme.set_color("bg_tab_active", "#222a38")
pinel.theme.set_color("bg_tab_inactive", "#161b24")
pinel.theme.set_color("bg_status_bar", "#101722")
pinel.theme.set_color("bg_tab_bar", "#0d131b")
pinel.theme.set_color("bg_hover", "#263042")
pinel.theme.set_color("bg_pressed", "#2d3950")
pinel.theme.set_color("bg_drag_handle", "#202938")

pinel.theme.set_color("text_primary", "#dbe7ff")
pinel.theme.set_color("text_secondary", "#b9c7e6")
pinel.theme.set_color("text_muted", "#93a0bd")
pinel.theme.set_color("text_dim", "#73809c")
pinel.theme.set_color("text_placeholder", "#5d6982")

pinel.theme.set_color("border_subtle", "#2a3445")
pinel.theme.set_color("border_very_subtle", "#1d2533")

pinel.theme.set_color("selection", "#7aa2f755")
pinel.theme.set_color("shadow_dark", "#00000088")
pinel.theme.set_color("shadow_light", "#7aa2f722")
]]
pinel.theme.set_color("bg_status_bar", "#101722")

--[[ Editor canvas
Below are the options specifically for modifying the editor canvas
The first paramter of the call is the actual attribute you would like to change and the second paramter is its new HEX calue

pinel.theme.set_color("editor.background", "#0f141c")
pinel.theme.set_color("editor.text_color", "#dbe7ff")
pinel.theme.set_color("editor.gutter_background", "#141b26")
pinel.theme.set_color("editor.gutter_border", "#222c3c")
pinel.theme.set_color("editor.line_number_color", "#6f7d97")
pinel.theme.set_color("editor.scrollbar_background", "#0f141c")
pinel.theme.set_color("editor.scroller_color", "#7aa2f7")
pinel.theme.set_color("editor.current_line_highlight", "#7aa2f733")
]]
pinel.theme.set_color("editor.current_line_highlight", "#7aa2f733")

pinel.ui.show_sidebar(true) -- coukd be either true or false depending on whether or not the sidebar should be shown on startup

-- Sidebar width ranges from 20.0 to 120.0
-- You do not need to have the decimal
pinel.ui.set_sidebar_width(220)
