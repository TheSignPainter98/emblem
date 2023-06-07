package.path = table.concat({
	package.path,
	dep_dir .. '/luacheck/src/?.lua',
	dep_dir .. '/luacheck/src/?/init.lua',
}, ';')

local yue = require('yue')
local luacheck = require('luacheck')

local function show(v)
	local t = type(v)
	if t == 'boolean' or t == 'nil' or t == 'number' then
		return tostring(v)
	elseif t == 'function' or t == 'userdata' or t == 'thread' then
		return '(' .. tostring(v) .. ')'
	elseif t == 'string' then
		return "'" .. v .. "'"
	elseif t ~= 'table' then
		error('unknown type ' .. t .. ' encountered')
	end

	local is_list = true
	local maxk = -1
	for k, _ in pairs(v) do
		if type(k) ~= 'number' then
			is_list = false
			break
		end
		if maxk < k then
			maxk = k
		end
	end
	is_list = is_list and maxk == #v

	if is_list then
		local buf = { '[' }
		for i = 1, #v do
			if i > 1 then
				buf[#buf + 1] = ', '
			end
			buf[#buf + 1] = show(v[i])
		end
		buf[#buf + 1] = ']'
		return table.concat(buf)
	end

	local buf = { '{' }
	for k, v2 in pairs(v) do
		if #buf > 2 then
			buf[#buf + 1] = ', '
		end
		buf[#buf + 1] = show(k)
		buf[#buf + 1] = ': '
		buf[#buf + 1] = show(v2)
	end
	buf[#buf + 1] = '}'
	return table.concat(buf)
end

local function die(msg)
	if msg then
		print('===============================================================')
		if type(msg) ~= 'string' then
			msg = show(msg)
		end
		print(msg)
		print('===============================================================')
	end
	os.exit(1)
end

local function dedent(string)
	local lines = {}
	local indent
	for line in string.gmatch(string, '([^\r\n]*)[\r\n]?') do
		if not indent then
			indent = '^' .. string.match(line, '^%s+')
		end
		lines[#lines + 1] = string.gsub(line, indent, '')
	end
	return table.concat(lines, '\n')
end

local function prepare(inputs, test)
	local prepared = {}
	for module, raw in pairs(inputs) do
		if test then
			prepared[module] = table.concat({
				[[macro spec = (t) ->                                      ]],
				[[	lines = {                                              ]],
				[[		'table.insert __tests, ->\n',                      ]],
				[[		'	import "busted" as :assert, :describe, :it\n', ]],
				[[		'	test = ',                                      ]],
				[[	}                                                      ]],
				[[	first = true                                           ]],
				[[	for line in t\gmatch '([^\r\n]*)[\r\n]?'               ]],
				[[		if first                                           ]],
				[[			first = false                                  ]],
				[[		else                                               ]],
				[[			lines[] = '\t\t'                               ]],
				[[		lines[] = line                                     ]],
				[[		lines[] = '\n'                                     ]],
				[[	lines[] = '\ttest!'                                    ]],
				[[	table.concat lines                                     ]],
				raw
			}, '\n')
		else
			-- TODO(kcza): simplify once yuescript supports empty macro output
			prepared[module] = 'macro spec = (t) -> "(->)!"\n' .. raw
		end
	end
	return prepared
end

local function asts_of(inputs)
	local asts = {}

	for module, raw in pairs(inputs) do
		print("generating AST of " .. module)
		local ast, err = yue.to_ast(raw)
		if err then
			die(err)
		end
		asts[module] = ast
	end

	return asts
end

local function luas_of(inputs)
	local luas = {}
	for module, raw in pairs(inputs) do
		print("generating lua code of " .. module)
		local lua, err, globals = yue.to_lua(raw, {
			implicit_return_root = true,
			reserve_line_number = true,
			reserve_comment = true,
			lint_global = true,
			module = module,
			target = '5.1',
		})
		if err then
			die(err)
		end
		luas[module] = sanitise(lua)
	end
	return luas
end

function sanitise(lua)
	return lua:gsub('%(function%(%)[ \t]*end%)%(%) %-%- [0-9]+[\r\n]+', '\n')
end

function dfs(name, arcs, handlers, stack)
	if stack == nil then
		stack = {}
	end

	for i = 1, #stack do
		if name == stack[i] then
			local on_cycle = handlers.on_cycle
			if on_cycle ~= nil then
				on_cycle(stack)
			end
			return
		end
	end

	stack[#stack + 1] = name

	local explore_node = handlers.explore_node
	if explore_node ~= nil then
		if explore_node(name) == false then
			stack[#stack] = nil
			return
		end
	end

	local nexts = arcs[name]
	if nexts then
		for i = 1, #nexts do
			dfs(nexts[i], arcs, handlers, stack)
		end
	end
	stack[#stack] = nil
end

local function assert_cyclefree(luas)
	local in_arcs = {}
	for name, lua in pairs(luas) do
		local ins = {}
		for line in string.gmatch(lua, '([^\r\n]*)[\r\n]') do
			local r = string.match(line, 'require%(?[\'"]([^\'")]*)[\'"]%)?')
			if r then
				ins[#ins + 1] = r
			end
		end
		in_arcs[name] = ins
	end

	local seen = {}
	for name in pairs(luas) do
		dfs(name, in_arcs, {
			explore_node = function(name)
				if seen[name] then
					return false
				end
				seen[name] = true
				return true
			end,
			on_cycle = function(stack)
				if test and #stack == 1 then
					-- Ignore modules importing themselves (helps testing,
					-- misuse should be obvious enough)
					return
				end
				local min_idx = 1
				local min = stack[1]
				for i = 2, #stack do
					local curr = stack[i]
					if curr < min then
						min = curr
						min_idx = i
					end
				end

				local pretty_cycle = { min }
				local curr_idx = min_idx + 1
				while curr_idx ~= min_idx do
					pretty_cycle[#pretty_cycle + 1] = stack[curr_idx]

					if curr_idx > #stack then
						curr_idx = 1
					else
						curr_idx = curr_idx + 1
					end
				end
				pretty_cycle[#pretty_cycle + 1] = pretty_cycle[1]

				die('cycle detected:\n\t' .. table.concat(pretty_cycle, '\n\t -> '))
			end,
		})
	end
end

local function lint(module, lua, test)
	local options = {
		globals = {
			em = {
				fields = {
					version = {
						fields = {
							major = {},
							minor = {},
							patch = {},
						},
					},
					cmds = {
						other_fields = true,
					},
					args = {},
					attrs = {},
					error = {},
					warn = {},
					observe = {},
				},
			},
		},
	}

	if test then
		options.globals.assert = { other_fields = true }
		options.globals.describe = {}
		options.globals.it = {}
	end

	local ignore = {
		['121'] = true,
		['122'] = true,
		['612'] = true,
	}

	if test then
		options.globals.__tests = {}
	end

	local report = luacheck.get_report(lua)
	local issues = luacheck.process_reports({ report }, options)[1]

	local messages = {}
	for i = 1, #issues do
		local issue = issues[i]
		local msg = luacheck.get_message(issue)

		if not ignore[issue.code] then
			messages[#messages + 1] = string.format(
				'luacheck: error[%d]: %s:%d:%d-%d: %s',
				issue.code,
				module,
				issue.line,
				issue.column,
				issue.end_column,
				msg
			)
		end
	end
	if #messages > 0 then
		die(table.concat(messages, '\n'))
	end
end

local function check(_asts, luas, test)
	for module, lua in pairs(luas) do
		lint(module, lua, test)
	end
end

local function encode(luas, test)
	local buf = {}

	if test then
		buf[#buf + 1] = 'local __tests = {}\n'
	end

	local is_global = {
		utf8 = true,
	}

	local modules = {}
	for name, _ in pairs(luas) do
		modules[#modules + 1] = name
	end
	table.sort(modules, function(a, b)
		local global_a = is_global[a]
		local global_b = is_global[b]
		if global_a ~= global_b then
			return global_a
		else
			return a < b
		end
	end)

	for i = 1, #modules do
		local module = modules[i]
		buf[#buf + 1] = 'package.preload["'
		buf[#buf + 1] = module
		buf[#buf + 1] = '"] = function()\n'
		buf[#buf + 1] = luas[module]
		buf[#buf + 1] = 'end\n'

		if is_global[module] then
			buf[#buf + 1] = 'require("'
			buf[#buf + 1] = module
			buf[#buf + 1] = '")\n'
		end
	end

	if test then
		buf[#buf + 1] = 'package.preload["__luatest"] = function()\n'
		buf[#buf + 1] = '\treturn function()\n'
		for i = 1, #modules do
			buf[#buf + 1] = '\t\trequire("' .. modules[i] .. '")\n'
		end
		buf[#buf + 1] = string.format([[
				package.loaders[#package.loaders + 1] = function(module)
					local pkg, mod = module:match("^([^.]+)%%.(.+)$")
					if pkg then
						local mod_file_name = mod:gsub('%%.', '/')
						local init = loadfile(string.format("%s", mod_file_name))
						if init then
							return init
						end
						return loadfile(string.format("%s", mod_file_name))
					end
				end
			]],
			string.format('%s/luassert/src/%%s/init.lua', dep_dir),
			string.format('%s/luassert/src/%%s.lua', dep_dir)
		)
		local path_parts = {
			'busted/?.lua',
			'busted/?/init.lua',
			'penlight/lua/?.lua',
			'penlight/lua/?/init.lua',
			'lua-term/?.lua',
			'lua-term/?/init.lua',
			'mediator_lua/src/?.lua',
			'lua_cliargs/src/?.lua',
			'lua_cliargs/src/?/init.lua',
			'?/src/init.lua',
			-- 'luassert/src/?.lua', -- Handled above
			'say/src/?.lua',
			'say/src/?/init.lua',
		}
		assert(#path_parts >= 1)
		buf[#buf + 1] = '\t\tpackage.path = table.concat({'
		for _, path_part in ipairs(path_parts) do
			buf[#buf + 1] = string.format('\n\t\t\t"%s/%s",', dep_dir, path_part)
		end
		buf[#buf + 1] = '\n\t\t}, ";")\n'
		buf[#buf + 1] = '\t\trequire("busted.runner")()\n'
		buf[#buf + 1] = '\t\tfor i = 1, #__tests do\n'
		buf[#buf + 1] = '\t\t\t__tests[i]()\n'
		buf[#buf + 1] = '\t\tend\n'
		buf[#buf + 1] = '\tend\n'
		buf[#buf + 1] = 'end\n'
	end

	local code = table.concat(buf)
	if not test then
		print("preprocessing code to bytecode")
		code = string.dump(load(code), true)
	end

	return { string.byte(code, 1, #code) }
end

local function compile(inputs, test)
	local inputs = prepare(inputs, test)

	local luas = luas_of(inputs)
	local asts = asts_of(inputs)

	assert_cyclefree(luas)
	check(asts, luas, test)

	return encode(luas, test)
end

local bytecode = compile(inputs, test)
print("lua generation done")
return bytecode
