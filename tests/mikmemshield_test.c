#include <check.h>

#include "miknet/mikmemmock.h"
#include "miknet/mikmemshield.h"

START_TEST(initialize)
{
	int *ptr;
	mikmemshield_t shield = mikmemshield_initialize();

	ptr = shield.calloc(1, sizeof(*ptr));
	*ptr = 80;

	ptr = shield.realloc(ptr, sizeof(*ptr) * 2);
	ptr[1] = 90;

	ck_assert(ptr[0] == 80);
	ck_assert(ptr[1] == 90);

	shield.free(ptr);
}
END_TEST

START_TEST(initialize_from)
{
	int a;
	mikmemshield_t shield = mikmemshield_initialize_from(
							mik_mock_calloc,
							mik_mock_realloc,
							mik_mock_free);

	mik_mock_calloc_set_return(&a);
	mik_mock_realloc_set_return(&a);

	ck_assert(shield.calloc(1, 1) == &a);
	ck_assert(shield.realloc(NULL, 1) == &a);
}
END_TEST

Suite *mikmemshield_suite()
{
	Suite *suite = suite_create("mikmemshield_suite");
	TCase *standard_use = tcase_create("mikmemshield");

	tcase_add_test(standard_use, initialize);
	tcase_add_test(standard_use, initialize_from);
	suite_add_tcase(suite, standard_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikmemshield = mikmemshield_suite();
	SRunner *runner = srunner_create(mikmemshield);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
