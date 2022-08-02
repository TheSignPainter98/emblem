---
-- @file std.func
-- @brief Provides functional abstractions
-- @author Edward Jones
-- @date 2021-09-17

import create, resume, wrap, yield from coroutine
import insert from table

func = {}

---
-- @brief A function which does nothing
func.do_nothing = ->

---
-- @brief Identity function, takes a value and returns it
-- @param x Value to return
-- @return `x`
func.id = (x) -> x

---
-- @brief Construct a list of keys from a table
-- @param t A table from which to extract keys
-- @return A list of keys in `t`
func.key_list = (t) -> [ k for k,_ in pairs t ]

---
-- @brief Construct a list of values in a table
-- @param t A table from which to extract values
-- @return A list of the values in `v`
func.value_list = (t) -> [ v for _,v in pairs t ]

---
-- @brief Creates a coroutine which yields the keys of a table
-- @param t A table from which to extract keys
-- @return A coroutine which yields the keys of `t`
func.keys = (t) ->
	wrap ->
		for k, _ in pairs t
			yield k

---
-- @brief Creates a coroutine which yields the values of a table
-- @param t A table from which to extract values
-- @return A coroutine which yields the values of `t`
func.values = (t) ->
	switch type t
		when 'function', 'coroutine'
			t
		else
			wrap ->
				for _, v in pairs t
					yield v

---
-- @brief Creates a coroutine which yields {k,v} pairs of a table
-- @param t A table from which to extract key-value pairs
-- @return A coroutine which yields the kv-pairs of `t`
func.kv_pairs = (t) ->
	wrap ->
		for k,v in pairs t
			yield {k,v}

---
-- @brief Creates a coroutine whihc filters the values of a coroutine by a predicate
-- @param p A predicate
-- @param es A coroutine which yields values to be filtered
-- @return A coroutine which yields the values of `e` of `es` which satisfy `p(e)` in the order they are yielded from `es`
func.filter = (p, es) ->
	wrap -> yield v for v in es when p v

---
-- @brief Returns a list of values of a given list which satisfy a predicate
-- @param p A predicate
-- @param es A list of values
-- @return A list of elements `e` of `es` which satisfy `p(e)`
func.filter_list = (p, es) -> [ e for e in *es when p e ]

---
-- @brief Maps the values yielded by a coroutine with a given function
-- @param f The mapping function
-- @param es The values to map
-- @return a coroutine which yields values `f(e)` for each `e` yielded from `es`
func.map = (f, es) ->
	wrap -> yield f v for v in es

---
-- @brief Takes values from a coroutine whilst a predicate holds
-- @param p Predicate to check
-- @param es From which to take values
-- @return A coroutine which yields values `e` of `es` for which `p(e)` holds, until the first which does not (at which point the coroutine finishes
func.take = (p, es) ->
	wrap ->
		for v in es
			if p v
				yield v
			else
				return

---
-- @brief Creates a list from the values yielded from a coroutine
-- @param c Coroutine from which to extract values
-- @return A list of values returned from `c`
func.co_to_list = (c) ->
	ret = {}
	for v in c
		insert ret, v
	ret

---
-- @brief Constructs a table from a coroutine which yields {k,v} pairs
-- @param c Coroutine from which to extract kv-pairs
-- @return A table `t` such that for each `{k,v}` yielded from `c`, `t[k]` = `v`
func.co_to_table = (c) ->
	ret = {}
	for {k,v} in c
		ret[k] = v
	ret

---
-- @brief Creates a coroutine which yields the values of a sequence
-- @param first The first value of the sequence
-- @param last The last value of the sequence
-- @param step The difference between consecutive yielded values
-- @return A coroutine which yields values starting from `first`, in increments of `step` until `last` is reached
func.seq = (first, last, step) ->
	wrap -> yield i for i = first, last, step

---
-- @brief Creates a coroutine which yields the natural numbers
-- @return A coroutine which yields the natural numbers
func.nat = ->
	i = 0
	wrap ->
		while true
			yield i
			i += 1

---
-- @brief Creates a coroutine which yields whole numbers
-- @return A coroutine which yields the whole numbers
func.whole = -> wrap -> yield i + 1 for i in func.nat!

---
-- @brief Creates a coroutine which yields the integers
-- @return a coroutine which yields the integers
func.int = ->
	i = 0
	wrap ->
		while true
			yield -i
			i += 1
			yield i

func
