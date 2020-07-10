#ifndef LIST_H_
#define LIST_H_

#include <stdbool.h>
#include <stdlib.h>

typedef struct DequeNode_s {
	void* data;
	struct DequeNode_s* nxt;
	struct DequeNode_s* prv;
} DequeNode;

typedef struct {
	DequeNode* fst;
	DequeNode* lst;
} Deque;

bool deque_create(Deque* dq);
bool deque_destroy(Deque* dq);
bool deque_append(Deque* dq, void* val);
bool deque_prepend(Deque* dq, void* val);
DequeNode* deque_pop_first(Deque* dq);
DequeNode* deque_pop_last(Deque* dq);
bool deque_empty(Deque* dq);

Deque deque_fmap(Deque* dq, void* (*fcn)(void*));
void deque_ifmap(Deque* dq, void* (*fcn)(void*));
Deque deque_filter(Deque* dq, bool (*fcn)(void*));
void* deque_foldr(Deque* dq, void* (*op)(void*, void*), void* initial);
void* deque_foldl(Deque* dq, void* (*op)(void*, void*), void* initial);

#endif /*  LIST_H_ */
