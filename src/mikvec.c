#include <miknet/miknet.h>

mikvec_t mikvec (mikpack_t data)
{
	mikvec_t vector = {0};
	vector.total_size = 1;
	vector.size = 1;
	vector.memsize = 1;
	vector.index = 0;
	vector.rs_mall = 0;
	vector.data = mik_try_alloc(vector.data, sizeof(mikpack_t));
	memcpy(vector.data, &data, sizeof(mikpack_t));

	return vector;
}

mikvec_t mikvec_add (mikvec_t vector, mikpack_t data)
{
	if (!vector.memsize)
		return mikvec(data);

	if (vector.size >= vector.memsize) {
		vector.memsize *= 2;
		size_t byte_size = vector.memsize * sizeof(mikpack_t);
		vector.data = mik_try_alloc(vector.data, byte_size);
		vector.rs_mall = 0;
		vector.total_size = vector.size + 1;
	}

	vector.data[vector.size] = data;
	vector.size++;

	return vector;
}

mikpack_t *mikvec_next (mikvec_t *vector)
{
	if (vector->index >= vector->size)
		return NULL;

	return &vector->data[vector->index++];
}

mikvec_t mikvec_clear (mikvec_t vector)
{
	int divisor = vector.rs_mall ? vector.rs_mall : 1;

	/* If the memory has gone unused for MIK_MEMEXP rounds, free it. */
	if ((vector.total_size / divisor) < (vector.memsize / 2)) {
		if (vector.rs_mall > MIK_MEMEXP) {
			vector.memsize /= 2;
			size_t byte_size = vector.memsize * sizeof(mikpack_t);
			vector.data = mik_try_alloc(vector.data, byte_size);
		}
	}

	int i;
	for (i = 0; i < vector.size; ++i)
		if (vector.data[i].data)
			free(vector.data[i].data);

	vector.total_size += vector.size;
	vector.size = 0;
	vector.index = 0;

	return vector;
}

mikvec_t mikvec_close (mikvec_t vector)
{
	mikvec_clear(vector);

	free(vector.data);
	
	vector.size = 0;
	vector.total_size = 0;
	vector.memsize = 0;
	vector.index = 0;
	vector.rs_mall = 0;
	vector.data = NULL;

	return vector;
}
