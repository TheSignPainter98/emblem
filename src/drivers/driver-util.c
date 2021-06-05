#include "driver-util.h"

#include <string.h>
#include <time.h>

#define TIME_STR_MAX_SUPPORTED_SIZE 26

void get_time_str(Str* time_str)
{
	time_t curr_time;
	time(&curr_time);
	char* time_buf		 = malloc(TIME_STR_MAX_SUPPORTED_SIZE * sizeof(char));
	struct tm* time_info = localtime(&curr_time);
	asctime_r(time_info, time_buf);
	time_buf[strlen(time_buf) - 1] = '\0'; // Strip trailing newline

	make_strr(time_str, time_buf);
}
