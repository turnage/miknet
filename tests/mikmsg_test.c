#include <check.h>
#include <stdint.h>

#include "miknet/mikmsg.h"
#include "miknet/mikdef.h"
#include "miknet/mikgram.h"

START_TEST(test_mikmsg)
{
	mikaddr_t addr;
	mikgram_t *gram = mikgram("Hello", 5);
	mikmsg_t *msg;

	/* Proper use. */
	msg = mikmsg(gram, &addr);
	ck_assert(msg != NULL);
	ck_assert(msg->data != gram->data);
	ck_assert(msg->data != NULL);
	ck_assert_int_eq(msg->len, mikgram_check(gram));
	ck_assert_int_eq(memcmp("Hello", msg->data, 5), 0);
	ck_assert_int_eq(memcmp(&msg->addr, &addr, sizeof(mikaddr_t)), 0);

	/* Bad inputs. */
	ck_assert_int_eq(mikmsg(NULL, &addr), NULL);
	ck_assert_int_eq(mikmsg(gram, NULL), NULL);
}
END_TEST

Suite *mikmsg_suite()
{
	Suite *suite = suite_create("mikmsg_suite");
	TCase *mikmsg_units = tcase_create("mikmsg_units");

	tcase_add_test(mikmsg_units, test_mikmsg);
	suite_add_tcase(suite, mikmsg_units);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikmsg = mikmsg_suite();
	SRunner *runner = srunner_create(mikmsg);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
