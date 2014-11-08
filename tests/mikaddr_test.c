#include <check.h>
#include <stdint.h>

#include "miknet/mikaddr.h"
#include "miknet/mikdef.h"
#include "testing/miksysmock.h"

START_TEST(test_create)
{
	mikaddr_t addr;
	struct addrinfo *expected_arg_val = (struct addrinfo *)700;
	posix_mock_t mock;
	int status;

	mock.posix = mikposixmock();
	mock.getaddrinfo_return = MIKERR_NONE;
	mock.getaddrinfo_arg_set = expected_arg_val;
	status = mikaddr(&addr, (posix_t *)&mock, "127.0.0.1", 80);

	ck_assert_int_eq(status, MIKERR_NONE);
	ck_assert_int_eq(addr.candidates, expected_arg_val);
}
END_TEST

START_TEST(test_create_sys_fails)
{
	mikaddr_t addr;
	posix_mock_t mock;
	int status;

	mock.posix = mikposixmock();
	mock.getaddrinfo_return = MIKERR_LOOKUP;
	status = mikaddr(&addr, (posix_t *)&mock, "127.0.0.1", 80);

	ck_assert_int_eq(status, MIKERR_LOOKUP);
}
END_TEST

START_TEST(test_create_bad_ptr)
{
	mikaddr_t addr;
	posix_mock_t mock;
	int status;

	mock.getaddrinfo_return = MIKERR_NONE;

	status = mikaddr(&addr, (posix_t *)&mock, NULL, 80);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikaddr(&addr, NULL, "127.0.0.1", 80);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikaddr(NULL, (posix_t *)&mock, "127.0.0.1", 80);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);
}
END_TEST

Suite *mikaddr_suite()
{
	Suite *suite = suite_create("mikaddr_suite");
	TCase *standard_use = tcase_create("mikaddr_standard_use");
	TCase *incorrect_use = tcase_create("mikaddr_incorrect_use");

	tcase_add_test(standard_use, test_create);
	tcase_add_test(standard_use, test_create_sys_fails);
	tcase_add_test(incorrect_use, test_create_bad_ptr);
	suite_add_tcase(suite, standard_use);
	suite_add_tcase(suite, incorrect_use);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikaddr = mikaddr_suite();
	SRunner *runner = srunner_create(mikaddr);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
