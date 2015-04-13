#include <check.h>
#include <stdint.h>

#include "miknet/miktime.h"
#include "testing/miksysmock.h"

START_TEST(test_miktime)
{
	uint64_t a;
	uint64_t b;

	a = miktime();
	b = miktime();

	ck_assert(a < b);
}
END_TEST

START_TEST(test_miktime_sleep)
{
	uint64_t a;
	uint64_t b;
	uint64_t remainder;

	a = miktime();
	remainder = miktime_sleep(1000);
	b = miktime();

	ck_assert(a < b - 1000 + remainder);
}
END_TEST

Suite *miktime_suite()
{
	Suite *suite = suite_create("miktime_suite");
	TCase *miktime_units = tcase_create("miktime_units");

	tcase_add_test(miktime_units, test_miktime);
	tcase_add_test(miktime_units, test_miktime_sleep);
	suite_add_tcase(suite, miktime_units);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *miktime = miktime_suite();
	SRunner *runner = srunner_create(miktime);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
