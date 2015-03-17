#include <check.h>
#include <stdint.h>

#include "miknet/mikaddr.h"
#include "miknet/mikdef.h"
#include "testing/miksysmock.h"

START_TEST(test_create)
{
	mikaddr_t addr;
	struct addrinfo expected_addr;
	struct sockaddr expected_sockaddr;
	struct sockaddr_in *expected;
	struct sockaddr_in *actual;
	posix_mock_t mock;
	int status;

	expected_addr.ai_addr = &expected_sockaddr;
	expected = (struct sockaddr_in *)expected_addr.ai_addr;

	mock.posix = mikposixmock();
	mock.getaddrinfo_return = MIKERR_NONE;
	mock.getaddrinfo_arg_set = &expected_addr;
	status = mikaddr(&addr, (posix_t *)&mock, "127.0.0.1", 80);
	actual = (struct sockaddr_in *)&addr.addr;

	ck_assert_int_eq(status, MIKERR_NONE);
	ck_assert_int_eq(addr.addrlen, expected_addr.ai_addrlen);
	ck_assert_int_eq(expected->sin_port, actual->sin_port);
	ck_assert_int_eq(expected->sin_addr.s_addr, actual->sin_addr.s_addr);

	/* NULL address should request INADDR_ANY */
	status = mikaddr(&addr, (posix_t *)&mock, NULL, 80);
	actual = (struct sockaddr_in *)&addr.addr;
	ck_assert_int_eq(status, MIKERR_NONE);
	ck_assert_int_eq(actual->sin_addr.s_addr, INADDR_ANY);
}
END_TEST

START_TEST(test_create_sys_fails)
{
	mikaddr_t addr;
	struct addrinfo expected_addr;
	posix_mock_t mock;
	int status;

	/* Failure by report. */
	mock.posix = mikposixmock();
	mock.getaddrinfo_return = MIKERR_LOOKUP;
	mock.getaddrinfo_arg_set = &expected_addr;
	status = mikaddr(&addr, (posix_t *)&mock, "127.0.0.1", 80);

	ck_assert_int_eq(status, MIKERR_LOOKUP);

	/* Failure by no results. */
	mock.getaddrinfo_return = MIKERR_NONE;
	mock.getaddrinfo_arg_set = NULL;
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
