#include <check.h>
#include <stdint.h>

#include "miknet/mikaddr.h"
#include "miknet/mikdef.h"
#include "testing/miksysmock.h"

START_TEST(test_mikaddr)
{
	mikaddr_t addr;
	struct addrinfo expected_addr;
	struct sockaddr expected_sockaddr;
	struct sockaddr_in *expected;
	struct sockaddr_in *actual;
	posix_mock_t mock;

	expected_addr.ai_addr = &expected_sockaddr;
	expected = (struct sockaddr_in *)expected_addr.ai_addr;
	mock.posix = mikposixmock();
	mock.getaddrinfo_arg_set = &expected_addr;

	/* Proper use. */
	mock.getaddrinfo_return = MIKERR_NONE;
	ck_assert_int_eq(
		mikaddr(&addr, (mikposix_t *)&mock, "0.0.0.0", 80),
		MIKERR_NONE);
	ck_assert_int_eq(addr.addrlen, expected_addr.ai_addrlen);
	actual = (struct sockaddr_in *)&addr.addr;
	ck_assert_int_eq(actual->sin_port, expected->sin_port);
	ck_assert_int_eq(actual->sin_addr.s_addr, expected->sin_addr.s_addr);

	/* NULL address should request INADDR_ANY */
	ck_assert_int_eq(	mikaddr(&addr, (mikposix_t *)&mock, NULL, 80),
				MIKERR_NONE);
	actual = (struct sockaddr_in *)&addr.addr;
	ck_assert_int_eq(actual->sin_addr.s_addr, INADDR_ANY);

	/* Failure by report. */
	mock.posix = mikposixmock();
	mock.getaddrinfo_return = -1;
	ck_assert_int_eq(
		mikaddr(&addr, (mikposix_t *)&mock, "0.0.0.0", 80),
		MIKERR_NET_FAIL);

	/* Failure by no results. */
	mock.getaddrinfo_return = MIKERR_NONE;
	mock.getaddrinfo_arg_set = NULL;
	ck_assert_int_eq(
		mikaddr(&addr, (mikposix_t *)&mock, "0.0.0.0", 80),
		MIKERR_NET_FAIL);

	/* Bad inputs. */
	mock.getaddrinfo_return = MIKERR_NONE;
	ck_assert_int_eq(mikaddr(&addr, NULL, "0.0.0.0", 80), MIKERR_BAD_PTR);
	ck_assert_int_eq(
		mikaddr(NULL, (mikposix_t *)&mock, "0.0.0.0", 80),
		MIKERR_BAD_PTR);
}
END_TEST

Suite *mikaddr_suite()
{
	Suite *suite = suite_create("mikaddr_suite");
	TCase *mikaddr_units = tcase_create("mikaddr_units");

	tcase_add_test(mikaddr_units, test_mikaddr);
	suite_add_tcase(suite, mikaddr_units);

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
