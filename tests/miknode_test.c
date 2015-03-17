#include <check.h>
#include <stdint.h>

#include "miknet/miknode.h"
#include "miknet/mikdef.h"
#include "testing/miksysmock.h"

START_TEST(test_create)
{
	int expected_socket = 10;
	mikaddr_t addr;
	miknode_t *node;
	posix_mock_t mock;
	uint8_t expected_max_peers = 100;

	mock.posix = mikposixmock();
	mock.bind_return = 0;
	mock.socket_return = expected_socket;

	node = miknode_create(&mock.posix, &addr, 1024, expected_max_peers);
	ck_assert(node != NULL);
	ck_assert(node->peers != NULL);
	ck_assert_int_eq(node->max_peers, expected_max_peers);
	ck_assert_int_eq(node->sockfd, expected_socket);
}
END_TEST

START_TEST(test_create_sys_fails)
{
	mikaddr_t addr;
	miknode_t *node;
	posix_mock_t mock;

	mock.posix = mikposixmock();
	mock.socket_return = 10;
	mock.bind_return = 1;
	node = miknode_create(&mock.posix, &addr, 10, 10);
	ck_assert(node == NULL);

	mock.socket_return = -1;
	mock.bind_return = 0;
	node = miknode_create(&mock.posix, &addr, 10, 10);
	ck_assert(node == NULL);
}
END_TEST

START_TEST(test_create_bad_ptr)
{
	mikaddr_t addr;
	miknode_t *node;

	node = miknode_create(NULL, &addr, 10, 10);
	ck_assert(node == NULL);
}
END_TEST

Suite *miknode_suite()
{
	Suite *suite = suite_create("miknode_suite");
	TCase *standard_use = tcase_create("miknode_standard_use");
	TCase *incorrect_use = tcase_create("miknode_incorrect_use");

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
	Suite *miknode = miknode_suite();
	SRunner *runner = srunner_create(miknode);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
