/**
 * @file locked.c
 * @brief Provides a locking structure for accessing data-structures, designed to prevent race conditions
 * @author Edward Jones
 * @date 2021-09-17
 */
#include "locked.h"

#include "logs/logs.h"
#include <stdlib.h>

void make_locked(Locked* l, void* data)
{
	if (!(l->mutex_lock = malloc(sizeof(pthread_mutex_t))) || pthread_mutex_init(l->mutex_lock, NULL))
	{
		log_err("Failed to initialise mutex lock");
		exit(1);
	}
	l->data = data;
}

void dest_locked(Locked* l, Destructor ed)
{
	pthread_mutex_destroy(l->mutex_lock);
	free(l->mutex_lock);
	if (ed)
		ed(l->data);
}

void* lock(Locked* l)
{
	pthread_mutex_lock(l->mutex_lock);
	return l->data;
}

void unlock(Locked* l) { pthread_mutex_unlock(l->mutex_lock); }
