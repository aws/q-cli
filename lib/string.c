#include "fig.h"
#include <ctype.h>

char *ltrim(char *s) {
  while (isspace(*s))
    s++;
  return s;
}

char *rtrim(char *s) {
  for (size_t i = strlen(s); i >= 0; i--) {
    if (i == 0 || !isspace(s[i - 1])) {
      s[i] = '\0';
      break;
    }
  }
  return s;
}

// https://stackoverflow.com/questions/1634359/is-there-a-reverse-function-for-strstr
char *strrstr(const char *haystack, const char *needle, const size_t haylen,
              const size_t needlelen) {
  if (*needle == '\0')
    return (char *)haystack;

  if (needlelen > haylen)
    return NULL;

  const char *p = haystack + haylen - needlelen;
  for (;;) {
    if (memcmp(p, needle, needlelen) == 0)
      return (char *)p;
    if (p == haystack)
      return NULL;
    --p;
  }
}

// Splice out everything between (start, end) in str.
void splicestr(char* str, const char* start, const char* end) {
  int pre_len;
  int post_len;
  char* splice_end;
  char* splice_start;
  const char* post_start;
  size_t str_len = strlen(str);
  size_t end_len = end == NULL ? 0 : strlen(end);

  if (start != NULL && (splice_start = strstr(str, start)) != NULL) {
    pre_len = splice_start - str;
  } else {
    pre_len = 0;
  }

  if (end != NULL && (splice_end = strrstr(str, end, str_len, end_len)) != NULL) {
    post_start = splice_end + end_len;
    post_len = str_len - (post_start - str);
  } else {
    post_start = str;
    post_len = 0;
  }

  char* tmpbuf = malloc(sizeof(char) * (pre_len + 1));
  sprintf(tmpbuf, "%.*s", pre_len, str);
  sprintf(str, "%.*s%.*s", pre_len, tmpbuf, post_len, post_start);
  free(tmpbuf);
}
