#include <check.h>
#include <stdint.h>

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

Suite *mikpack_suite()
{
	Suite *suite = suite_create("mikpack_suite");
	TCase *standard_use = tcase_create("mikpack");

	tcase_add_test(standard_use, memory_est);
	suite_add_tcase(suite, standard_use);

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
