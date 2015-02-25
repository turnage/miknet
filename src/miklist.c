#include <stdlib.h>

#include <miknet/miklist.h>

static void append(miklist_t *list, miklist_t *node)
{
	miklist_t *nav;

	for (nav = list; nav->next != NULL; nav = nav->next);

	nav->next = node;
}

miklist_t *miklist_enqueue(miklist_t *list, mikpack_t *payload)
{
	miklist_t *new_node;

	if (payload == NULL)
		return list;

	new_node = malloc(sizeof(miklist_t));
	new_node->next = NULL;
	new_node->payload = payload;

	if (list == NULL)
		list = new_node;
	else
		append(list, new_node);

	return list;
}

const mikpack_t *miklist_peek(const miklist_t *list)
{
	if (list == NULL)
		return NULL;

	return list->payload;
}

miklist_t *miklist_dequeue(miklist_t *list)
{
	miklist_t *new_front;

	if (list == NULL)
		return NULL;

	new_front = list->next;

	free(list->payload);
	free(list);

	return new_front;
}
