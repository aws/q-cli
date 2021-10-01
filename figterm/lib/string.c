#include "fig.h"
#include <ctype.h>

char *ltrim(char *s) {
  while (isspace(*s))
    s++;
  return s;
}

void replace(char *s, char o, char n) {
  while (*s != '\0') {
    if (*s == o)
      *s = n;
    s++;
  }
}

char *rtrim(char *s, int min) {
  for (size_t i = strlen(s); i >= 0; i--) {
    if (i == 0 || i == min || !isspace(s[i - 1])) {
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
