#include <miknet/miknet.h>

miklist_t *miklist (void *data, size_t len)
{
	miklist_t list, *head;
	list.next = NULL;
	list.len = len;
	list.data = data;

	head = calloc(1, sizeof(miklist_t));
	list.data = calloc(1, len);

	memcpy(head, &list, sizeof(miklist_t));
	memcpy(head->data, data, len);

	return head;
}

miklist_t *miklist_add (miklist_t *head, void *data, size_t len)
{
	if (!head)
		return miklist (data, len);

	miklist_t list, *i, *pos;
	list.next = NULL;
	list.len = len;

	for (i = head; i; i = i->next) {
		pos = i;
	}

	pos->next = calloc(1, sizeof(miklist_t));
	memcpy(pos->next, &list, sizeof(miklist_t));

	pos->data = calloc(1, len);
	memcpy(pos->data, data, len);

	return head;
}

miklist_t *miklist_next (miklist_t *head)
{
	miklist_t *new_head = head->next;

	free(head->data);
	free(head);

	return new_head;
}

void miklist_close (miklist_t *head)
{
	miklist_t *i, *pos;

	if (head->next) {
		pos = head;
		for (i = head->next; i; i = i->next) {

			if (pos->len == sizeof(mikcommand_t))
				free(((mikcommand_t *)pos->data)->pack.data);
			else if (pos->len == sizeof(mikevent_t))
				free(((mikevent_t *)pos->data)->pack.data);

			free(pos->data);
			free(pos);
			pos = i;
		}
		free(pos->data);
		free(pos);
	} else {
		free(head->data);
		free(head);
	}
}