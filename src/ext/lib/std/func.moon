import create, resume, wrap, yield from coroutine
import insert from table

func = {}

func.do_nothing = ->
func.id = (x) -> x

func.key_list = (t) -> [ k for k,_ in pairs t ]
func.value_list = (t) -> [ v for _,v in pairs t ]

func.keys = (t) ->
	wrap ->
		for k, _ in pairs t
			yield k

func.values = (t) ->
	wrap ->
		for _, v in pairs t
			yield v

func.filter = (p, es) ->
	wrap -> yield v for v in es when p v

func.map = (f, es) ->
	wrap -> yield f v for v in es

func.take = (p, es) ->
	wrap ->
		for v in es
			if p v
				yield v
			else
				return

func.co_to_list = (c) ->
	ret = {}
	for v in c
		insert ret, v
	ret

func.seq = (first, last, step) ->
	wrap -> yield i for i = first, last, step

func.nat = ->
	i = 0
	wrap ->
		while true
			yield i
			i += 1
func.whole = -> wrap -> yield i + 1 for i in func.nat!
func.int = ->
	i = 0
	wrap ->
		while true
			yield -i
			i += 1
			yield i

func
