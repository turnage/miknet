#include <check.h>
#include <stdint.h>

#include "miknet/mikgram.h"
#include "miknet/mikdef.h"

START_TEST(test_mikgram)
{
	char hello[] = "Hello";
	mikgram_t *gram;

	/* Proper use. */
	gram = mikgram(hello, 6);
	ck_assert(gram->data != NULL);
	ck_assert(gram->len > 6);
	ck_assert_int_eq(gram->next, NULL);
	ck_assert_int_eq(((uint8_t *)gram->data)[0], 6);
	ck_assert_int_eq(((uint8_t *)gram->data)[1], 0);

	/* Bad inputs. */
	ck_assert_int_eq(mikgram(NULL, 6), NULL);
	ck_assert_int_eq(mikgram(hello, SIZE_MAX), NULL);

	mikgram_close(gram);
}
END_TEST

START_TEST(test_mikgram_check)
{
	char hello[] = "Hello";
	mikgram_t *gram;
	mikgram_t nodata_gram;

	gram = mikgram(hello, 6);
	ck_assert(gram != NULL);

	/* Proper use. */
	gram->len = 1024;
	ck_assert_int_eq(mikgram_check(gram), 6);
	gram->len = 3;
	ck_assert_int_eq(mikgram_check(gram), MIKERR_BAD_VALUE);

	/* Bad inputs. */
	ck_assert_int_eq(mikgram_check(NULL), MIKERR_BAD_PTR);
	mikgram_close(gram);

	nodata_gram.data = NULL;
	ck_assert_int_eq(mikgram_check(&nodata_gram), MIKERR_BAD_PTR);
}
END_TEST

START_TEST(test_mikgram_extract)
{
	char hello[] = "Hello";
	char buffer[6] = {0};
	mikgram_t *gram;

	gram = mikgram(hello, 6);
	ck_assert(gram != NULL);

	/* Proper use. */
	ck_assert_int_eq(	mikgram_extract(gram, buffer, 6),
				MIK_SUCCESS);
	ck_assert_int_eq(memcmp(buffer, hello, 6), 0);

	/* Bad inputs. */
	ck_assert_int_eq(	mikgram_extract(gram, NULL, 10),
				MIKERR_BAD_PTR);
	ck_assert_int_eq(	mikgram_extract(NULL, buffer, 10),
				MIKERR_BAD_PTR);
	gram->len = 0;
	ck_assert_int_eq(	mikgram_extract(gram, buffer, 10),
				MIKERR_BAD_VALUE);
	gram->data = NULL;
	ck_assert_int_eq(	mikgram_extract(gram, buffer, 10),
				MIKERR_BAD_PTR);
	mikgram_close(gram);
}
END_TEST

Suite *mikgram_suite()
{
	Suite *suite = suite_create("mikgram_suite");
	TCase *gram_units = tcase_create("mikgram_units");

	tcase_add_test(gram_units, test_mikgram);
	tcase_add_test(gram_units, test_mikgram_check);
	tcase_add_test(gram_units, test_mikgram_extract);
	suite_add_tcase(suite, gram_units);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikgram = mikgram_suite();
	SRunner *runner = srunner_create(mikgram);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
