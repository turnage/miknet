#include <check.h>
#include <stdint.h>
#include <stdlib.h>

#include "miknet/miklist.h"

START_TEST(miklist_create)
{
	int data = 18;
	miklist_t *list;

	list = miklist_enqueue(NULL, &data);

	ck_assert(list != NULL);
	ck_assert_int_eq(*(int *)list->payload, 18);
}
END_TEST

START_TEST(miklist_create_bad)
{
	miklist_t *list;

	list = miklist_enqueue(NULL, NULL);

	ck_assert(list == NULL);
}
END_TEST

START_TEST(miklist_add)
{
	int data = 18;
	miklist_t *list;

	list = miklist_enqueue(NULL, &data);
	list = miklist_enqueue(list, &data);

	ck_assert_int_eq(*(int *)list->next->payload, 18);
}
END_TEST

START_TEST(miklist_remove)
{
	miklist_t *list;

	list = miklist_enqueue(NULL, malloc(sizeof(int)));
	list = miklist_enqueue(list, malloc(sizeof(int)));

	list = miklist_dequeue(list);
	ck_assert(list->next == NULL);

	list = miklist_dequeue(list);
	ck_assert(list == NULL);
}
END_TEST

Suite *miklist_suite()
{
	Suite *suite = suite_create("miklist_suite");
	TCase *standard_use = tcase_create("miklist");
	TCase *incorrect_use = tcase_create("miklist_incorrect");

	tcase_add_test(standard_use, miklist_create);
	tcase_add_test(standard_use, miklist_add);
	tcase_add_test(standard_use, miklist_remove);

	tcase_add_test(incorrect_use, miklist_create_bad);

	suite_add_tcase(suite, standard_use);
	suite_add_tcase(suite, incorrect_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *miklist = miklist_suite();
	SRunner *runner = srunner_create(miklist);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
