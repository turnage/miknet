#include <check.h>
#include <stdint.h>

#include "miknet/mikaddr.h"
#include "miknet/mikdef.h"
#include "testing/miksysmock.h"

START_TEST(test_create)
{
	mikaddr_t addr;
	uint64_t expected_arg_val = 700;
	posix_t posix = mikposixmock();
	int status;

	miksysmock_set_return(0);
	miksysmock_set_arg(expected_arg_val);
	status = mikaddr(&addr, &posix, "127.0.0.1", 80);

	ck_assert_int_eq(status, MIKERR_NONE);
	ck_assert_int_eq((uint64_t)addr.candidates, expected_arg_val);
}
END_TEST

START_TEST(test_create_sys_fails)
{
	mikaddr_t addr;
	posix_t posix = mikposixmock();
	int status;

	miksysmock_set_return(-1);
	status = mikaddr(&addr, &posix, "127.0.0.1", 80);

	ck_assert_int_eq(status, MIKERR_LOOKUP);
}
END_TEST

START_TEST(test_connect)
{
	mikaddr_t addr;
	posix_t posix = mikposixmock();
	struct addrinfo candidates;
	int status;

	miksysmock_set_return(0);
	candidates.ai_next = NULL;
	addr.hint.ai_family = AF_INET;
	addr.hint.ai_socktype = SOCK_STREAM;
	addr.candidates = &candidates;
	status = mikaddr_connect(&addr, &posix);

	ck_assert(status >= 0);
}
END_TEST

START_TEST(test_connect_sys_fails_socket)
{
	mikaddr_t addr;
	posix_t posix = mikposixmock();
	struct addrinfo candidates;
	int status;

	miksysmock_set_return(-1);
	candidates.ai_next = NULL;
	addr.hint.ai_family = AF_INET;
	addr.hint.ai_socktype = SOCK_STREAM;
	addr.candidates = &candidates;
	status = mikaddr_connect(&addr, &posix);

	ck_assert_int_eq(status, MIKERR_SOCKET);
}
END_TEST

START_TEST(test_connect_sys_fails_connect)
{
	mikaddr_t addr;
	posix_t posix = mikposixmock();
	struct addrinfo candidates;
	int status;

	miksysmock_set_return(2);
	candidates.ai_next = NULL;
	addr.hint.ai_family = AF_INET;
	addr.hint.ai_socktype = SOCK_STREAM;
	addr.candidates = &candidates;
	status = mikaddr_connect(&addr, &posix);

	ck_assert_int_eq(status, MIKERR_CONNECT);
}
END_TEST

START_TEST(test_create_bad_ptr)
{
	mikaddr_t addr;
	posix_t posix = mikposixmock();
	int status;

	miksysmock_set_return(0);

	status = mikaddr(&addr, &posix, NULL, 80);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikaddr(&addr, NULL, "127.0.0.1", 80);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	status = mikaddr(NULL, &posix, "127.0.0.1", 80);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);
}
END_TEST

START_TEST(test_connect_bad_ptr)
{
	mikaddr_t addr;
	posix_t posix = mikposixmock();
	struct addrinfo candidates;
	int status;

	miksysmock_set_return(0);
	candidates.ai_next = NULL;
	addr.hint.ai_family = AF_INET;
	addr.hint.ai_socktype = SOCK_STREAM;

	addr.candidates = &candidates;
	status = mikaddr_connect(NULL, &posix);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);

	addr.candidates = &candidates;
	status = mikaddr_connect(&addr, NULL);
	ck_assert_int_eq(status, MIKERR_BAD_PTR);
}
END_TEST

START_TEST(test_connect_bad_addr)
{
	mikaddr_t addr;
	posix_t posix = mikposixmock();
	int status;

	miksysmock_set_return(0);
	addr.hint.ai_family = AF_INET;
	addr.hint.ai_socktype = SOCK_STREAM;
	addr.candidates = NULL;

	status = mikaddr_connect(&addr, &posix);
	ck_assert_int_eq(status, MIKERR_BAD_ADDR);
}
END_TEST

Suite *mikaddr_suite()
{
	Suite *suite = suite_create("mikaddr_suite");
	TCase *standard_use = tcase_create("mikaddr_standard_use");
	TCase *incorrect_use = tcase_create("mikaddr_incorrect_use");

	tcase_add_test(standard_use, test_create);
	tcase_add_test(standard_use, test_create_sys_fails);
	tcase_add_test(standard_use, test_connect);
	tcase_add_test(standard_use, test_connect_sys_fails_socket);
	tcase_add_test(standard_use, test_connect_sys_fails_connect);
	tcase_add_test(incorrect_use, test_create_bad_ptr);
	tcase_add_test(incorrect_use, test_connect_bad_ptr);
	tcase_add_test(incorrect_use, test_connect_bad_addr);
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
