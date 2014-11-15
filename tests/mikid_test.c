#include <check.h>
#include <stdint.h>

#include "miknet/mikid.h"

START_TEST(generate_ids)
{
	uint16_t a = mikid();
	uint16_t b = mikid();

	ck_assert(a != b);
}
END_TEST

Suite *mikid_suite()
{
	Suite *suite = suite_create("mikid_suite");
	TCase *standard_use = tcase_create("mikid");

	tcase_add_test(standard_use, generate_ids);
	suite_add_tcase(suite, standard_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikid = mikid_suite();
	SRunner *runner = srunner_create(mikid);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
