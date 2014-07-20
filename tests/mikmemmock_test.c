#include <check.h>

#include "miknet/mikmemmock.h"

START_TEST(calloc_mock)
{
	int a = 70;
	int *ptr;

	mik_mock_calloc_set_return(&a);
	ck_assert(mik_mock_calloc(1, 1) == &a);

	mik_mock_calloc_use_system(MIK_TRUE);
	ptr = mik_mock_calloc(1, sizeof(*ptr));
	ck_assert(ptr != &a);

	*ptr = 90;
	ck_assert(*ptr == 90 && a != 90);

	mik_mock_free_use_system(MIK_TRUE);
	mik_mock_free(ptr);

	mikmemmock_reset();
}
END_TEST

START_TEST(realloc_mock)
{
	int a = 70;
	int *ptr;

	mik_mock_realloc_set_return(&a);
	ck_assert(mik_mock_realloc(NULL, 1) == &a);

	mik_mock_realloc_use_system(MIK_TRUE);
	ptr = mik_mock_realloc(NULL, sizeof(*ptr));
	ck_assert(ptr != &a);

	*ptr = 90;
	ck_assert(*ptr == 90 && a != 90);

	mik_mock_free_use_system(MIK_TRUE);
	mik_mock_free(ptr);

	mikmemmock_reset();
}
END_TEST

START_TEST(free_mock)
{
	mik_mock_free(NULL);

	mikmemmock_reset();
}
END_TEST

Suite *mikmemmock_suite()
{
	Suite *suite = suite_create("mikmemmock_suite");
	TCase *standard_use = tcase_create("mikmemmock");

	tcase_add_test(standard_use, calloc_mock);
	tcase_add_test(standard_use, realloc_mock);
	tcase_add_test(standard_use, free_mock);
	suite_add_tcase(suite, standard_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikmemmock = mikmemmock_suite();
	SRunner *runner = srunner_create(mikmemmock);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
