#pragma once

#include "destructor.h"
#include <pthread.h>

typedef struct
{
	pthread_mutex_t* mutex_lock;
	void* data;
} Locked;

#define USE_LOCK(decl, lockedObj, cmds)                                                                                \
	{                                                                                                                  \
		decl = lock(lockedObj);                                                                                        \
		cmds;                                                                                                          \
		unlock(lockedObj);                                                                                             \
	}

void make_locked(Locked* l, void* data) __attribute__((nonnull(1)));
void dest_locked(Locked* l, Destructor ed) __attribute__((nonnull(1)));

void* lock(Locked* l) __attribute__((nonnull(1)));
void unlock(Locked* l) __attribute__((nonnull(1)));
