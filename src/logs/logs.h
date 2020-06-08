#ifndef LOGS_H_
#define LOGS_H_

/**
 * @brief Write a warning to stderr
 *
 * @param format Warning format (printf)
 * @param ... Possible printf arguments
 */
void log_warn(const char* __restrict format, ...) __attribute__((format(printf, 1, 2)));
/**
 * @brief Write an error stderr
 *
 * @param format Error format (printf)
 * @param ... Possible printf arguments
 */
void log_err(const char* __restrict format, ...) __attribute__((cold)) __attribute__((format(printf, 1, 2)));
/**
 * @brief Write information to stderr
 *
 * @param format Information format (printf)
 * @param ... Possible printf arguments
 */
void log_info(const char* __restrict format, ...) __attribute__((format(printf, 1, 2)));
/**
 * @brief Write a success message to stderr
 *
 * @param format Success message format (printf)
 * @param ... Possible printf arguments
 */
void log_succ(const char* __restrict format, ...) __attribute__((format(printf, 1, 2)));

#endif /* LOGS_H_ */
