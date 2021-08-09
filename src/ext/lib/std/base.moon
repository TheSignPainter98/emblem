import concat from table

collectgarbage 'stop' -- TODO: remove the need for this!

base = { :eval, :eval_string, :include_file, :node_types, :requires_reiter }

class PublicTable
	__tostring: show
export em = PublicTable!
base.em = em

node_string = (n) ->
	if n == nil
		return nil
	if 'table' != type n
		return tostring n
	switch n.type
		when node_types.word
			return n.word
		when node_types.call
			return node_string n.result
		when node_types.content
			return concat [ node_string w for w in *n.content when w != nil ], ' '
		else
			error "Unrecognised node type '#{n.type}'"
			return 1
base.node_string = node_string

base.eval_string = (d) ->
	if 'userdata' == type d
		return node_string eval d
	tostring d

base.iter_num = -> em_iter

base
