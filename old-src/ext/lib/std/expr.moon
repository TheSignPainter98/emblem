---
-- @file std.expr
-- @brief Provides a framework for evaluating expressions
-- @author Edward Jones
-- @date 2021-09-25

import P, R, V from require 'lpeg'
import Directive, em, eval_string, is_instance, set_var_string from require 'std.base'
import id from require 'std.func'
import log_err, log_err_here, log_warn_here from require 'std.log'
import show from require 'std.show'

---
-- @brief Represents an operator to be parsed from an expression
class Operator
	new: (@name, @func) =>
		@pat = P @name

op_arity =
	UNARY: 1
	BINARY: 2
import UNARY, BINARY from op_arity

---
-- @brief Represents a set of operators which all have the same precidence, allowing them to be parsed at the same level in an expression tree
class PrecidenceGroup
	new: (@arity, @ops, @exit_cond) =>
		error "Cannot parse operation, is not an instance of Operator: #{op}" for op in *@ops when not is_instance Operator, op
		-- Op func table
		@op_funcs = { op.name,op.func for op in *@ops }
		-- Pattern for the group's operators
		@pat = @ops[1].pat
		for i = 2,#@ops
			@pat += @ops[i].pat
		@pat /= @op_funcs
	reductor: (x) => x
	exit_early: (v) =>
		return false unless @exit_cond
		@.exit_cond v

int_to_bool = (b) -> b != 0
bool_to_int = (b) ->
	return b if 'boolean' != type b
	return 1 if b
	0

---
-- @brief Represents an operator group which contains only unary operators
class UnaryPrecidenceGroup extends PrecidenceGroup
	new: (...) =>
		super UNARY, ...
	reductor: (args) =>
		{f,val} = args
		f val\eval!

---
-- @brief Represents a precidence group of binary operators which can be multiply-composed at the same level of an expression-treee
class MultiBinaryPrecidenceGroup extends PrecidenceGroup
	new: (...) =>
		super BINARY, ...
	reductor: (args) =>
		n = #args
		a = args[1]\eval!
		for i=2,n,2
			return a if @exit_early a
			f = args[i]
			b = args[i+1]\eval!
			a = f a, b
		a

---
-- @brief Represents a precidence group of binary comparators which can be multiply-composed at the same level of an expression-tree
class MultiBinaryComparisonPredicenceGroup extends PrecidenceGroup
	new: (...) =>
		super BINARY, ...
	reductor: (args) =>
		n = #args
		a = args[1]\eval!
		ret = true
		for i=2,n,2
			f = args[i]
			b = args[i+1]\eval!
			ret and= f a, b
			a = b
			break if ret == 0 or not ret
		bool_to_int ret

class ExprTree
	new: (@reductor, @parts) =>
	eval: => @.reductor @parts

mk_expr_tree = (reductor) -> (...) ->
	args = {...}
	return args[1] if #args == 1 and 'number' != type args[1]
	ExprTree reductor, args

class ExprTreeLeaf
	new: (@val) =>
	eval: => @val
mk_leaf = (...) -> ExprTreeLeaf ...

---
-- @brief List of precedence groups from which expression parsers are created
operators = {
	MultiBinaryComparisonPredicenceGroup {
		Operator '<==>', (a,b) -> bool_to_int a == b
		Operator '==>', (a,b) -> bool_to_int a == 0 or b != 0
		Operator '<==', (a,b) -> bool_to_int a != 0 or b == 0
	}
	MultiBinaryPrecidenceGroup {
		Operator '||', ((a,b) -> if a != 0 a else b)
	}, (a, ...) -> a != 0
	MultiBinaryPrecidenceGroup {
		Operator '&&', (a,b) -> if a == 0 a else b
	}, (a) -> a == 0
	MultiBinaryPrecidenceGroup {
		Operator '<=>', (a,b) -> bool_to_int a == b
		Operator '=>', (a,b) -> bool_to_int a == 0 or b != 0
	}
	MultiBinaryPrecidenceGroup {
		Operator '!=', (a,b) -> bool_to_int a != b
		Operator '==', (a,b) -> bool_to_int a == b
	}
	MultiBinaryComparisonPredicenceGroup {
		Operator '<=', (a,b) -> bool_to_int a <= b
		Operator '>=', (a,b) -> bool_to_int a >= b
		Operator '<', (a,b) -> bool_to_int a < b
		Operator '>', (a,b) -> bool_to_int a > b
	}
	MultiBinaryPrecidenceGroup {
		Operator '~', (a,b) -> (a & ~b) | (~a & b) -- Moonscript does not currently support the ~ xor operator.
		Operator '|', (a,b) -> a | b
	}
	MultiBinaryPrecidenceGroup {
		Operator '&', (a,b) -> a & b
	}
	MultiBinaryPrecidenceGroup {
		Operator '+', (a,b) -> a + b
		Operator '-', (a,b) -> a - b
	}
	MultiBinaryPrecidenceGroup {
		Operator '//', (a,b) ->
			return a // b unless b == 0
			a / b
		Operator '%', (a,b) ->
			return a % b unless b == 0
			a / b
		Operator '*', (a,b) -> a * b
		Operator '/', (a,b) -> a / b
	}
	UnaryPrecidenceGroup {
		Operator '!', (a) -> bool_to_int a == 0
		Operator '-', (a) -> -a
		Operator '~', (a) -> ~a
		Operator '+', (a) -> a
	}
	MultiBinaryPrecidenceGroup {
		Operator '^', (a,b) -> a ^ b
	}
}

---
-- @brief Represents a grammar for parsing expressions
class ExprGrammar
	new: (@prec_groups) =>
		error "Cannot parse precidence group, is not an instance of PrecidenceGroup: #{show getmetatable pg}, #{pg}" for pg in *@prec_groups when not is_instance PrecidenceGroup, pg
		@args = {}
		@grammar = {
			"R0",
			R0: (V 'R1') * '\0'
			Value:
				@bool! +
				@hex! +
				@bin! +
				@num! +
				@arg! +
				'(' * (V 'R1') * ')'
		}
		last_prec_group = @prec_groups[#@prec_groups]
		r = 0
		for pg in *@prec_groups
			r += 1
			prec_group_ops = { op.name, op.func for op in *pg.ops }
			rule_key = "R#{r}"
			next_rule_key = pg != last_prec_group and "R#{r+1}" or "Value"
			switch pg.arity
				when UNARY
					@grammar[rule_key] = (V next_rule_key) + pg.pat * (V rule_key) / mk_expr_tree pg\reductor
				when BINARY
					@grammar[rule_key] = (V next_rule_key) * (pg.pat * V next_rule_key)^0 / mk_expr_tree pg\reductor
				else
					log_err "Unknown arity #{pg.arity} for precidence group #{show pg}"
		@parser = P @grammar
	num: => (R('09')^1 * ('.' * (R '09')^1)^-1) / tonumber / mk_leaf
	bool: => ((P 'true') + P 'false') / @\toboolean / mk_leaf
	arg: => '\b' * ((R '09')^1 / @\get_arg / mk_leaf)
	bin: => '0b' * ((R '01')^1 / (@\to_based_num 2) / mk_leaf)
	hex: => '0x' * (((R '09') + R 'af')^1 / (@\to_based_num 16) / mk_leaf)
	to_based_num: (base) => (n) -> tonumber n, base
	eval: (s) =>
		ps, @args, s = @preprocess_parse_input s
		@args = {}
		expr_tree = @parser\match ps
		unless expr_tree
			log_warn_here "Failed to parse expression '#{s}'"
			return nil
		expr_tree\eval!
	preprocess_parse_input: (s) =>
		s = (eval_string s)\lower!\gsub ' ', ''
		s .. '\0', {}, s
	get_arg: (a) =>
		if arg = @args[tonumber a\sub 2]
			return @toval arg
		log_err_here "WARNING: Unknown variable reference: #{a}"
	toval: (s) => (tonumber s) or @toboolean s
	toboolean: (s) =>
		switch s\lower!
			when 'false', '0'
				0
			else
				1

---
-- @brief Takes a string (or document pointer), parses it and evaluates it as an expression
-- @param str The text to parse and evaluate
-- @return An integer which was evaluated from the expression represented by `str` or nil on failure
expr = (s) ->
	g = ExprGrammar operators
	g\eval s

em.expr = Directive 1, 0, "Parse and evaluate an expression", expr
em.set_var_expr = Directive 2, 0, "Set the value of a variable in the current scope, evaluating the value as an expression", (n,v) -> set_var_string n, (expr v), true
em.find_set_var_expr = Directive 2, 0, "Set the value of a variable in the current scope, evaluating the value as an expression", (n,v) -> set_var_string n, (expr v), true, true

{ :ExprGrammar, :MultiBinaryComparisonPredicenceGroup, :MultiBinaryPrecidenceGroup, :Operator, :PrecidenceGroup, :UnaryPrecidenceGroup, :expr, :operators }
