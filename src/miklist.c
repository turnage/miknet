#include <miknet/miknet.h>

miklist_t *miklist (ref *data)
{
	miklist_t *head = calloc(1, sizeof(miklist_t));
	memcpy(head, data, sizeof(miklist_t));

	return head;
}

miklist_t *miklist_add (miklist_t *head, ref *data)
{
	if (!head)
		return miklist (data);

	miklist_t *i = NULL;
	miklist_t *pos = NULL;

	for (i = head; i; i = i->next) {
		pos = i;
	}

	pos->next = calloc(1, sizeof(miklist_t));
	memcpy(pos->next, data, sizeof(miklist_t));

	return head;
}

miklist_t *miklist_next (miklist_t *head)
{
	if (!head)
		return NULL;

	miklist_t *new_head = head->next;

	free(head->pack.data);
	free(head);

	return new_head;
}

void miklist_close (miklist_t *head)
{
	while (head) {
		head = miklist_next(head);
	}
}