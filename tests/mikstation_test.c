#include <check.h>
#include <stdint.h>

#include "miknet/mikstation.h"

#include "miknet/mikdef.h"
#include "miknet/mikgram.h"
#include "testing/miksysmock.h"

START_TEST(test_mikstation_discard)
{
	posix_mock_t mock;
	int sockfd;

	mock.posix = mikposixmock();
	sockfd = 3;

	/* Proper use. */
	mock.recvfrom_return = 5;
	ck_assert_int_eq(mikstation_discard(sockfd, &mock.posix), 5);

	mock.recvfrom_return = -1;
	ck_assert_int_eq(	mikstation_discard(sockfd, &mock.posix),
				MIKERR_NET_FAIL);

	/* Bad inputs. */
	mock.recvfrom_return = 5;
	ck_assert_int_eq(mikstation_discard(sockfd, NULL), MIKERR_BAD_PTR);
	ck_assert_int_eq(mikstation_discard(-1, &mock.posix), MIKERR_BAD_VALUE);
}
END_TEST

START_TEST(test_mikstation_poll)
{
	posix_mock_t mock;
	int sockfd;

	mock.posix = mikposixmock();
	sockfd = 9;

	/* Proper use. */
	mock.recvfrom_return = 5;
	ck_assert_int_eq(mikstation_poll(sockfd, &mock.posix), 5);

	mock.recvfrom_return = -1;
	ck_assert_int_eq(mikstation_poll(sockfd, &mock.posix), MIKERR_NET_FAIL);

	/* Bad inputs. */
	mock.recvfrom_return = 5;
	ck_assert_int_eq(mikstation_poll(sockfd, NULL), MIKERR_BAD_PTR);
	ck_assert_int_eq(mikstation_poll(-1, &mock.posix), MIKERR_BAD_VALUE);
}
END_TEST

START_TEST(test_mikstation_receive)
{
	mikaddr_t addr;
	mikgram_t gram;
	posix_mock_t mock;
	int sockfd;

	mock.posix = mikposixmock();
	sockfd = 80;

	/* Proper use. */
	mock.recvfrom_return = 5;
	ck_assert_int_eq(
		mikstation_receive(sockfd, &mock.posix, &gram, &addr), 0);

	mock.recvfrom_return = -1;
	ck_assert_int_eq(
		mikstation_receive(sockfd, &mock.posix, &gram, &addr),
		MIKERR_NET_FAIL);

	/* Bad inputs. */
	mock.recvfrom_return = 5;
	ck_assert_int_eq(
		mikstation_receive(-2, &mock.posix, &gram, &addr),
		MIKERR_BAD_VALUE);
	ck_assert_int_eq(
		mikstation_receive(sockfd, NULL, &gram, &addr), MIKERR_BAD_PTR);
	ck_assert_int_eq(
		mikstation_receive(sockfd, &mock.posix, NULL, &addr),
		MIKERR_BAD_PTR);
	ck_assert_int_eq(
		mikstation_receive(sockfd, &mock.posix, &gram, NULL),
		MIKERR_BAD_PTR);
}
END_TEST

START_TEST(test_mikstation_send)
{
	mikaddr_t addr;
	char data[MIKNET_GRAM_MIN_SIZE];
	mikgram_t gram;
	posix_mock_t mock;
	int sockfd;

	mock.posix = mikposixmock();
	gram.data = data;
	gram.len = MIKNET_GRAM_MIN_SIZE;
	sockfd = 7;

	/* Proper use. */
	mock.sendto_return = MIKNET_GRAM_MIN_SIZE;
	ck_assert_int_eq(mikstation_send(sockfd, &mock.posix, &gram, &addr), 0);

	mock.sendto_return = MIKNET_GRAM_MIN_SIZE - 1;
	ck_assert_int_eq(
		mikstation_send(sockfd, &mock.posix, &gram, &addr),
		MIKERR_NET_FAIL);

	mock.sendto_return = -1;
	ck_assert_int_eq(
		mikstation_send(sockfd, &mock.posix, &gram, &addr),
		MIKERR_NET_FAIL);

	/* Bad inputs. */
	mock.sendto_return = MIKNET_GRAM_MIN_SIZE;
	ck_assert_int_eq(	mikstation_send(-1, &mock.posix, &gram, &addr),
				MIKERR_BAD_VALUE);
	ck_assert_int_eq(
		mikstation_send(sockfd, NULL, &gram, &addr), MIKERR_BAD_PTR);
	ck_assert_int_eq(
		mikstation_send(sockfd, &mock.posix, NULL, &addr),
		MIKERR_BAD_PTR);
	ck_assert_int_eq(
		mikstation_send(sockfd, &mock.posix, &gram, NULL),
		MIKERR_BAD_PTR);

	gram.data = NULL;
	ck_assert_int_eq(
		mikstation_send(sockfd, &mock.posix, &gram, &addr),
		MIKERR_BAD_PTR);
}
END_TEST

Suite *mikstation_suite()
{
	Suite *suite = suite_create("mikstation_suite");
	TCase *mikstation_units = tcase_create("mikstation_units");

	tcase_add_test(mikstation_units, test_mikstation_discard);
	tcase_add_test(mikstation_units, test_mikstation_poll);
	tcase_add_test(mikstation_units, test_mikstation_receive);
	tcase_add_test(mikstation_units, test_mikstation_send);
	suite_add_tcase(suite, mikstation_units);

	return suite;
}

int main(int argc, char **argv)
{
	int failure_count;
	Suite *mikstation = mikstation_suite();
	SRunner *runner = srunner_create(mikstation);

	srunner_run_all(runner, CK_NORMAL);
	failure_count = srunner_ntests_failed(runner);
	srunner_free(runner);

	return failure_count;
}
