#include <check.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "miknet/mikdef.h"
#include "miknet/mikmeta.h"
#include "miknet/mikpack.h"

START_TEST(make_short_packet)
{
	char data[6] = "Hello";
	const size_t length = 6;
	mikmeta_t metadata;
	mikpack_t *pack;
	int status;

	status = mikpack(&pack, MIK_SAFE, (uint8_t *)data, length);
	ck_assert_int_eq(status, MIKERR_NONE);

	status = mikpack_frag(pack, 0, &metadata);
	ck_assert_int_eq(status, MIKERR_NONE);

	ck_assert_int_eq(metadata.size, length);
	ck_assert_int_eq(metadata.part, 0);
	ck_assert_int_eq(metadata.type, MIK_SAFE);
	ck_assert_int_eq(pack->frags, 1);
	ck_assert_int_eq(pack->ref_count, 0);
	ck_assert_int_eq(memcmp(mikpack_frag_data(pack, 0), "Hello", 6), 0);

	mikpack_close(pack);
}
END_TEST

START_TEST(make_long_packet)
{
	char data[(MIKPACK_FRAG_SIZE * 2) - 100] = {0};
	const size_t length = (MIKPACK_FRAG_SIZE * 2) - 100;
	mikmeta_t metadata;
	mikpack_t *pack;
	int status;

	status = mikpack(&pack, MIK_UNSAFE, (uint8_t *)data, length);
	ck_assert_int_eq(status, MIKERR_NONE);

	status = mikpack_frag(pack, 1, &metadata);
	ck_assert_int_eq(status, MIKERR_NONE);

	ck_assert_int_eq(metadata.size, MIKPACK_FRAG_SIZE - 100);
	ck_assert_int_eq(metadata.part, 1);
	ck_assert_int_eq(metadata.type, MIK_UNSAFE);
	ck_assert_int_eq(pack->frags, 2);
	ck_assert_int_eq(pack->ref_count, 0);

	mikpack_close(pack);
}
END_TEST

START_TEST(make_packet_bad_ptr)
{
	uint8_t num;
	mikpack_t *pack;
	int status;

	status = mikpack(NULL, MIK_SAFE, &num, 1);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikpack(&pack, MIK_SAFE, NULL, 1);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikpack(&pack, MIK_SAFE, &num, 0);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);
}
END_TEST

START_TEST(mikpack_frag_bad_ptr)
{
	mikpack_t stackpack;
	mikpack_t *pack = &stackpack;
	mikmeta_t metadata;
	int status;

	status = mikpack_frag(pack, 0, NULL);
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
