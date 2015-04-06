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

	/* Proper use. */
	node = miknode_create(&mock.posix, &addr, 1024, expected_max_peers);
	ck_assert(node != NULL);
	ck_assert(node->peers != NULL);
	ck_assert_int_eq(node->max_peers, expected_max_peers);
	ck_assert_int_eq(node->sockfd, expected_socket);

	/* System failures. */
	mock.socket_return = 10;
	mock.bind_return = 1;
	ck_assert(miknode_create(&mock.posix, &addr, 10, 10) == NULL);

	mock.socket_return = -1;
	mock.bind_return = 0;
	ck_assert(miknode_create(&mock.posix, &addr, 10, 10) == NULL);

	/* Bad inputs. */
	node = miknode_create(NULL, &addr, 10, 10);
	ck_assert(node == NULL);
}
END_TEST

START_TEST(test_insert_peer)
{
	miknode_t node = {0};
	mikpeer_t peers[3] = {{0}};
	mikaddr_t addr;

	node.peers = peers;
	node.max_peers = 3;

	/* Proper use. */
	miknode_insert_peer(&node, &addr);
	miknode_insert_peer(&node, &addr);
	ck_assert_int_eq(node.peers[0].exists, MIK_TRUE);
	ck_assert_int_eq(node.peers[1].exists, MIK_TRUE);
	ck_assert_int_eq(node.peers[2].exists, MIK_FALSE);
}
END_TEST

Suite *miknode_suite()
{
	Suite *suite = suite_create("miknode_suite");
	TCase *miknode_units = tcase_create("miknode_units");

	tcase_add_test(miknode_units, test_create);
	tcase_add_test(miknode_units, test_insert_peer);
	suite_add_tcase(suite, miknode_units);

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
