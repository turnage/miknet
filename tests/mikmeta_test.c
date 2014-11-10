#include <check.h>
#include <stdint.h>

#include "miknet/mikmeta.h"

START_TEST(serialize)
{
	uint8_t serialized[3] = {0};
	mikmeta_t metadata = {0};
	int status;

	metadata.id = 0xaabb;
	metadata.part = 0xccdd;
	metadata.type = MIK_JOIN;
	metadata.size = 0xeeff;
	status = mikmeta_serialize(&metadata, serialized);

	ck_assert_int_eq(status, 0);
	ck_assert_int_eq(serialized[0], 0xaa);
	ck_assert_int_eq(serialized[1], 0xbb);
	ck_assert_int_eq(serialized[2], 0xcc);
	ck_assert_int_eq(serialized[3], 0xdd);
	ck_assert_int_eq(serialized[4], MIK_JOIN);
	ck_assert_int_eq(serialized[5], 0xee);
	ck_assert_int_eq(serialized[6], 0xff);

}
END_TEST

START_TEST(serialize_bad_ptr)
{
	mikmeta_t metadata = {0};
	uint8_t serialized[3] = {0};

	ck_assert_int_eq(mikmeta_serialize(NULL, NULL), -1);
	ck_assert_int_eq(mikmeta_serialize(&metadata, NULL), -1);
	ck_assert_int_eq(mikmeta_serialize(NULL, serialized), -1);
}
END_TEST

START_TEST(deserialize)
{
	uint8_t serialized[3] = {0};
	mikmeta_t deserialized = {0};

	serialized[0] = 0xaa;
	serialized[1] = 0xbb;
	serialized[2] = 0xcc;
	serialized[3] = 0xdd;
	serialized[4] = MIK_DATA;
	serialized[5] = 0xee;
	serialized[6] = 0xff;

	deserialized = mikmeta_deserialize(serialized);

	ck_assert_int_eq(deserialized.type, MIK_DATA);
	ck_assert_int_eq(deserialized.id, 0xaabb);
	ck_assert_int_eq(deserialized.part, 0xccdd);
	ck_assert_int_eq(deserialized.size, 0xeeff);
}
END_TEST

START_TEST(deserialize_bad_ptr)
{
	mikmeta_t deserialized;

	deserialized = mikmeta_deserialize(NULL);

	ck_assert_int_eq(deserialized.type, MIK_NONE);
}
END_TEST

Suite *mikmeta_suite()
{
	Suite *suite = suite_create("mikmeta_suite");
	TCase *standard_use = tcase_create("mikmeta_standard");
	TCase *incorrect_use = tcase_create("mikmeta_incorrect");

	tcase_add_test(standard_use, serialize);
	tcase_add_test(standard_use, deserialize);
	tcase_add_test(incorrect_use, serialize_bad_ptr);
	tcase_add_test(incorrect_use, deserialize_bad_ptr);
	suite_add_tcase(suite, standard_use);
	suite_add_tcase(suite, incorrect_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikmeta = mikmeta_suite();
	SRunner *runner = srunner_create(mikmeta);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
