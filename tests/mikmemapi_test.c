#include <check.h>

#include "miknet/mikmemapi.h"

START_TEST(normal_calls)
{
	char *test = mik_calloc(1, sizeof(*test));
	*test = 'c';
	ck_assert(*test == 'c');

	test = mik_realloc(test, sizeof(*test));
	*(test + 1) = 'b';
	ck_assert(*(test + 1) == 'b');

	mik_free(test);
}
END_TEST

Suite *mikmemapi_suite()
{
	Suite *suite = suite_create("mikmemapi_suite");
	TCase *standard_use = tcase_create("mikmemapi");

	tcase_add_test(standard_use, normal_calls);
	suite_add_tcase(suite, standard_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikmemapi = mikmemapi_suite();
	SRunner *runner = srunner_create(mikmemapi);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
