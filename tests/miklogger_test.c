#include <check.h>
#include <stdint.h>
#include <string.h>

#include "miknet/miklogger.h"

START_TEST(normal_calls)
{
	char buffer[1024] = {0};
	int expected_int = 5;
	char expected_char = 'k';
	uint16_t expected_unsigned = 450;

	mik_log_core(MIK_LOG_INFO, buffer, "test %d.", expected_int);
	ck_assert_str_eq(buffer, "INFO: test 5.");
	memset(buffer, 0, sizeof(buffer));

	mik_log_core(MIK_LOG_TRIP, buffer, "test %c.", expected_char);
	ck_assert_str_eq(buffer, "TRIPPING: test k.");
	memset(buffer, 0, sizeof(buffer));

	mik_log_core(MIK_LOG_FATAL, buffer, "test %u.", expected_unsigned);
	ck_assert_str_eq(buffer, "FATAL: test 450.");
	memset(buffer, 0, sizeof(buffer));
}
END_TEST

START_TEST(logging_null)
{
	char buffer[1024] = {0};

	mik_log_core(MIK_LOG_INFO, buffer, NULL);
	ck_assert_str_eq(buffer, "TRIPPING: Attempted to log NULL.\n");
}
END_TEST

START_TEST(logging_off)
{
	char empty[1024] = {0};
	char buffer[1024] = {0};

	mik_log_toggle(MIK_LOG_OFF);
	mik_log_core(MIK_LOG_INFO, buffer, "Anyone home?");
	ck_assert(memcmp(buffer, empty, 1024) == 0);

	mik_log_toggle(MIK_LOG_ON);
	mik_log_core(MIK_LOG_INFO, buffer, "Anyone home?");
	ck_assert_str_eq(buffer, "INFO: Anyone home?");
}
END_TEST

Suite *miklogger_suite()
{
	Suite *suite = suite_create("miklogger_suite");
	TCase *standard_use = tcase_create("mik_log");
	TCase *incorrect_use = tcase_create("mik_log_incorrect");

	tcase_add_test(standard_use, normal_calls);
	tcase_add_test(incorrect_use, logging_null);
	tcase_add_test(incorrect_use, logging_off);
	suite_add_tcase(suite, standard_use);
	suite_add_tcase(suite, incorrect_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *miklogger = miklogger_suite();
	SRunner *runner = srunner_create(miklogger);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
