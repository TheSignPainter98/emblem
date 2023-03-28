-- package.path = package.path .. ';crates/yuescript/luacheck/src/?/init.lua'
package.path = package.path .. ';/home/kcza/Documents/projects/emblem/crates/yuescript/luacheck/src/?/init.lua'
package.path = package.path .. ';/home/kcza/Documents/projects/emblem/crates/yuescript/luacheck/src/?.lua'

local yue = require('yue')
local luacheck = require('luacheck')

-- input:
--     find files
--     figure out dependencies and hence load order
function compile(module_name, raw, test)
	local script
	if test then
		script = 'macro tests = (t) -> t\n' .. raw
	else
		script = 'macro tests = (t) -> ""\n' .. raw
	end

	local lua = yuescript_to_lua(module_name, script)

	lint(module_name, lua)

	-- TODO(kcza): statically check with luacheck
	-- TODO(kcza): typecheck with teal?

	if test then
		return encode(lua)
	else
		return encode(string.dump(load(lua)))
	end
end

function yuescript_to_lua(name, script)
	local lua, err, globals = yue.to_lua(script, {
		implicit_return_root = true,
		reserve_line_number = true,
		reserve_comment = true,
		lint_global = true,
		module = name,
		target = '5.1',
	})
	if err then
		error(err)
	end
	return lua
end

function lint(module_name, lua)
	local options = {
		globals = {
			em = {
				fields = {
					cmds = {
						other_fields = true,
					},
					args = {},
					attrs = {},
					error = {},
					warn = {},
					observe = {},
				}
			}
		}
	}

	local report = luacheck.get_report(lua)
	local issues = luacheck.process_reports({report}, options)[1]

	if #issues > 0 then
		print("===============================================================")
	end

	for i = 1, #issues do
		local issue = issues[i]
		local msg = luacheck.get_message(issue)

		print(string.format("error[%d]: %s:%d:%d-%d: %s", issue.code, module_name, issue.line, issue.column, issue.end_column, msg))
	end

	if #issues > 0 then
		print("===============================================================")
		error("problems were found")
	end
end

function encode(lua)
	return { string.byte(lua, 1, #lua) }
end

return compile(module_name, raw, test)
