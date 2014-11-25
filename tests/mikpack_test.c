#include <check.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "miknet/mikdef.h"
#include "miknet/mikmeta.h"
#include "miknet/mikpack.h"

START_TEST(memory_est)
{
	size_t data_a = mikpack_mem_est(800);
	size_t data_b = mikpack_mem_est(800);
	size_t empty = mikpack_mem_est(0);

	/* Results should be deterministic. */
	ck_assert_int_eq(data_a, data_b);

	/* The size should be larger than the data size to account for
           metadata. */
	ck_assert(data_a > 800);
	ck_assert(empty > 0);
}
END_TEST

START_TEST(make_short_packet)
{
	char data[6] = "Hello";
	uint8_t *dest;
	const size_t length = 6;
	mikmeta_t metadata;
	mikpack_t pack;
	int status;

	dest = calloc(1, mikpack_mem_est(length));
	status = mikpack(&pack, 0, (uint8_t *)data, length, dest);
	ck_assert_int_eq(status, MIKERR_NONE);

	status = mikpack_frag(&pack, 0, &metadata);
	ck_assert_int_eq(status, MIKERR_NONE);

	ck_assert_int_eq(metadata.size, length);
	ck_assert_int_eq(metadata.part, 0);
	ck_assert_int_eq(metadata.type, MIK_DATA);
	ck_assert_int_eq(pack.flags, 0);
	ck_assert_int_eq(pack.frags, 1);
	ck_assert_int_eq(pack.ref_count, 0);
	ck_assert_int_eq(pack.data, dest);
	ck_assert_int_eq(memcmp(mikpack_frag_data(&pack, 0), "Hello", 6), 0);

	free(dest);
}
END_TEST

START_TEST(make_long_packet)
{
	char data[(MIKPACK_FRAG_SIZE * 2) - 100] = {0};
	uint8_t *dest;
	const size_t length = (MIKPACK_FRAG_SIZE * 2) - 100;
	mikmeta_t metadata;
	mikpack_t pack;
	int status;

	dest = calloc(1, mikpack_mem_est(length));
	status = mikpack(&pack, 0, (uint8_t *)data, length, dest);
	ck_assert_int_eq(status, MIKERR_NONE);

	status = mikpack_frag(&pack, 1, &metadata);
	ck_assert_int_eq(status, MIKERR_NONE);

	ck_assert_int_eq(metadata.size, MIKPACK_FRAG_SIZE - 100);
	ck_assert_int_eq(metadata.part, 1);
	ck_assert_int_eq(metadata.type, MIK_DATA);
	ck_assert_int_eq(pack.frags, 2);
	ck_assert_int_eq(pack.ref_count, 0);
	ck_assert_int_eq(pack.data, dest);

	free(dest);
}
END_TEST

START_TEST(make_packet_bad_ptr)
{
	uint8_t num;
	mikpack_t pack;
	int status;

	status = mikpack(NULL, 0, &num, 1, &num);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikpack(&pack, 0, NULL, 1, &num);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikpack(&pack, 0, &num, 0, &num);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikpack(&pack, 0, &num, 1, NULL);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);
}
END_TEST

START_TEST(mikpack_frag_bad_ptr)
{
	mikpack_t pack;
	mikmeta_t metadata;
	int status;

	status = mikpack_frag(&pack, 0, NULL);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);
	
	status = mikpack_frag(NULL, 0, &metadata);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);
}
END_TEST

START_TEST(mikpack_frag_bad_arg)
{
	mikpack_t pack;
	mikmeta_t metadata;
	int status;

	pack.frags = 3;
	status = mikpack_frag(&pack, 4, &metadata);
	ck_assert_int_eq(status, MIKERR_NO_SUCH_FRAG);
}
END_TEST

START_TEST(mikpack_frag_data_bad_ptr)
{
	uint8_t *data;

	data = mikpack_frag_data(NULL, 0);
	ck_assert_int_eq(data, NULL);
}
END_TEST

START_TEST(mikpack_frag_data_bad_arg)
{
	mikpack_t pack;
	uint8_t *data;

	pack.frags = 4;
	data = mikpack_frag_data(&pack, 5);
	ck_assert_int_eq(data, NULL);
}
END_TEST

Suite *mikpack_suite()
{
	Suite *suite = suite_create("mikpack_suite");
	TCase *standard_use = tcase_create("mikpack");
	TCase *incorrect_use = tcase_create("mikpack_incorrect");

	tcase_add_test(standard_use, memory_est);
	tcase_add_test(standard_use, make_short_packet);
	tcase_add_test(standard_use, make_long_packet);

	tcase_add_test(incorrect_use, make_packet_bad_ptr);
	tcase_add_test(incorrect_use, mikpack_frag_bad_ptr);
	tcase_add_test(incorrect_use, mikpack_frag_bad_arg);
	tcase_add_test(incorrect_use, mikpack_frag_data_bad_ptr);
	tcase_add_test(incorrect_use, mikpack_frag_data_bad_arg);

	suite_add_tcase(suite, standard_use);
	suite_add_tcase(suite, incorrect_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikpack = mikpack_suite();
	SRunner *runner = srunner_create(mikpack);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
