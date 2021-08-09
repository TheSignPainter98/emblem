import show from require 'std.show'
import is_list from require 'std.util'
import insert from table

{
	word: word_type,
	content: content_type,
	call: call_type
} = node_types

class Node
	new: (@type) =>
	__tostring: => show @

sanitise_concat_input = (x) ->
	return {} if x == nil
	return {x} if ('table' != type x) or x.type == word_type or x.type == call_type
	return x.content if x.type == content_type
	error "Unrecognised concatenation input: #{show x}"

local Content
concat_ast_nodes = (as, bs) ->
	as = sanitise_concat_input as
	bs = sanitise_concat_input bs
	newlist = [ a for a in *as ]
	insert newlist, b for b in *bs
	Content newlist

class Word extends Node
	new: (@word) => super word_type
	__concat: concat_ast_nodes

class Content extends Node
	new: (@content) => super content_type
	__concat: concat_ast_nodes

class Call extends Node
	new: (@name, args) =>
		super call_type
		if is_list args
			@args = args
		else
			@args = {args}
	__concat: concat_ast_nodes
	__shl: (c, a) ->
		if 'table' != type c or c.type != call_type
			error "Left operand to an argument-append must be a call, instead got #{show c}"
		newargs = [ arg for arg in *c.args ]
		insert newargs, a
		Call c.name, newargs

mkcall = (name) -> (args) -> Call name, args

-- w = Word 'heyo'
-- x = Word 'worldo'
-- cs = Content { 1, 2, 3, 4 }
-- ds = Content { 'a', 'b', 'c', 'd' }
-- em['test-func'] = -> Content { "hello,", "world", "this", "is", "a", "list!" }
-- em['test-func'] = -> Content { (Word "hello,"), Word "world" }
-- em['test-func'] = -> Content [ Word n for n in *{1, 2, 3, 4} ]
-- em['test-func'] = -> Call 'bf', "hello!"
-- em['test-func'] = -> (Word "Heyo,") .. "there!"
-- em['test-func'] = -> "hello" .. Word "world!"
-- em['test-func'] = -> 'hello' .. ((Call 'bf', "boldened") << "world!") .. 'and this comes afterwards'

{ :Call, :Content, :Word, :mkcall }
