#include "fig.h"
#include "vterm.h"

enum { COLOR_TYPE_NONE, COLOR_TYPE_NAMED, COLOR_TYPE_RGB };

// Internal color representation, similar to fish representation and agnostic
// to vterm library.
typedef struct {
  uint8_t type;
  uint8_t name_idx;
  uint8_t rgb[3];
} Color;

bool bool_from_string(const char* x) {
  return x != NULL && strchr("YTyt1", x[0]) != NULL;
}

// Updates our idea of whether we support term256 and term24bit (see issue #10222).
color_support_t get_color_support() {
  // Detect or infer term256 support. If fish_term256 is set, we respect it;
  // otherwise infer it from the TERM variable or use terminfo.
  bool support_term256 = false;
  bool support_term24bit = false;

  char* term = getenv("TERM");
  char* fish_term256 = getenv("fish_term256");

  if (fish_term256 != NULL) {
    support_term256 = bool_from_string(fish_term256);
  } else if (term != NULL && strstr(term, "256color") != NULL) {
    support_term256 = true;
  } else if (term != NULL && strstr(term, "xterm") != NULL) {
    // Assume that all 'xterm's can handle 256, except for Terminal.app from Snow Leopard
    char* term_program = getenv("TERM_PROGRAM");
    if (term_program != NULL && strcmp("Apple_Terminal", term_program) == 0) {
      char* tpv = getenv("TERM_PROGRAM_VERSION");
      if (tpv != NULL && strtod(tpv, NULL) > 299) {
        support_term256 = true;
      }
    } else {
      support_term256 = true;
    }
  } else if (false /* cur_term != NULL && max_colors >= 256 */) {
    // TODO(sean): check portability of curses.
    support_term256 = true;
  }

  char* ct;
  char* it;
  char* vte;
  char* fish_term24bit = getenv("fish_term24bit");
  // Handle $fish_term24bit
  if (fish_term24bit != NULL) {
    support_term24bit = bool_from_string(fish_term24bit);
  } else if (getenv("STY") != NULL || (term != NULL && strncmp("eterm", term, 5) == 0)) {
    // Screen and emacs' ansi-term swallow truecolor sequences,
    // so we ignore them unless force-enabled.
    support_term24bit = false;
  } else if (false /*cur_term != NULL && max_colors >= 32767 */) {
    // TODO(sean): check portability of curses.
    // $TERM wins, xterm-direct reports 32767 colors, we assume that's the minimum
    // as xterm is weird when it comes to color.
    support_term24bit = true;
  } else if ((ct = getenv("COLORTERM")) != NULL) {
    // If someone set $COLORTERM, that's the sort of color they want.
    if (strcmp(ct, "truecolor") == 0 || strcmp(ct, "24bit") == 0) {
      support_term24bit = true;
    }
  } else if (getenv("KONSOLE_VERSION") != NULL || getenv("KONSOLE_PROFILE_NAME") != NULL) {
    // All konsole versions that use $KONSOLE_VERSION are new enough to support this,
    // so no check is necessary.
    support_term24bit = true;
  } else if ((it = getenv("ITERM_SESSION_ID")) != NULL) {
    // Supporting versions of iTerm include a colon here.
    // We assume that if this is iTerm, it can't also be st, so having this check
    // inside is okay.
    if (strchr(it, ':') != NULL) {
      support_term24bit = true;
    }
  } else if (term != NULL && strncmp("st-", term, 3) == 0) {
    support_term24bit = true;
  } else if ((vte = getenv("VTE_VERSION")) != NULL) {
    if (strtod(vte, NULL) > 3600) {
      support_term24bit = true;
    }
  }

  return (support_term256 ? color_support_term256 : 0)
    | (support_term24bit ? color_support_term24bit : 0);
}

/// Compare wide strings with simple ASCII canonicalization.
/// \return -1, 0, or 1 if s1 is less than, equal to, or greater than s2, respectively.
static int simple_icase_compare(const char *s1, const char *s2) {
  for (size_t idx = 0; s1[idx] || s2[idx]; idx++) {
    char c1 = s1[idx];
    char c2 = s2[idx];

    // "Canonicalize" to lower case.
    if (L'A' <= c1 && c1 <= L'Z') c1 = L'a' + (c1 - L'A');
    if (L'A' <= c2 && c2 <= L'Z') c2 = L'a' + (c2 - L'A');

    if (c1 != c2) {
      return c1 < c2 ? -1 : 1;
    }
  }
  // We must have equal lengths and equal values.
  return 0;
}

static int parse_hex_digit(wchar_t x) {
  switch (x) {
    case L'0': {
                 return 0x0;
               }
    case L'1': {
                 return 0x1;
               }
    case L'2': {
                 return 0x2;
               }
    case L'3': {
                 return 0x3;
               }
    case L'4': {
                 return 0x4;
               }
    case L'5': {
                 return 0x5;
               }
    case L'6': {
                 return 0x6;
               }
    case L'7': {
                 return 0x7;
               }
    case L'8': {
                 return 0x8;
               }
    case L'9': {
                 return 0x9;
               }
    case L'a':
    case L'A': {
                 return 0xA;
               }
    case L'b':
    case L'B': {
                 return 0xB;
               }
    case L'c':
    case L'C': {
                 return 0xC;
               }
    case L'd':
    case L'D': {
                 return 0xD;
               }
    case L'e':
    case L'E': {
                 return 0xE;
               }
    case L'f':
    case L'F': {
                 return 0xF;
               }
    default: {
               return -1;
             }
  }
}

static unsigned long squared_difference(long p1, long p2) {
  long diff = labs(p1 - p2);
  return diff * diff;
}

static uint8_t convert_color(const uint8_t rgb[3], const uint32_t *colors, size_t color_count) {
  long r = rgb[0], g = rgb[1], b = rgb[2];
  unsigned long best_distance = -1;
  uint8_t best_index = -1;
  for (size_t idx = 0; idx < color_count; idx++) {
    uint32_t color = colors[idx];
    long test_r = (color >> 16) & 0xFF, test_g = (color >> 8) & 0xFF,
         test_b = (color >> 0) & 0xFF;
    unsigned long distance = squared_difference(r, test_r) + squared_difference(g, test_g) +
      squared_difference(b, test_b);
    if (distance <= best_distance) {
      best_index = idx;
      best_distance = distance;
    }
  }
  return best_index;
}

Color *try_parse_rgb(const char* name) {
  // We support the following style of rgb formats (case insensitive):
  //  #FA3, #F3A035, FA3, F3A035
  size_t digit_idx = 0, len = strlen(name);

  // Skip any leading #.
  if (len > 0 && name[0] == '#') digit_idx++;

  bool success = false;
  size_t i;
  Color* color = malloc(sizeof(Color));
  color->type = COLOR_TYPE_RGB;
  if (len - digit_idx == 3) {
    // Format: FA3
    for (i = 0; i < 3; i++) {
      int val = parse_hex_digit(name[digit_idx++]);
      if (val < 0) break;
      color->rgb[i] = val * 16 + val;
    }
    success = (i == 3);
  } else if (len - digit_idx == 6) {
    // Format: F3A035
    for (i = 0; i < 3; i++) {
      int hi = parse_hex_digit(name[digit_idx++]);
      int lo = parse_hex_digit(name[digit_idx++]);
      if (lo < 0 || hi < 0) break;
      color->rgb[i] = hi * 16 + lo;
    }
    success = (i == 3);
  }
  if (!success) {
    free(color);
    return NULL;
  } 
  return color;
}

struct named_color_t {
  const char* name;
  uint8_t idx;
  uint8_t rgb[3];
};

// Keep this sorted alphabetically
static struct named_color_t named_colors[] = {
  {"black", 0, {0x00, 0x00, 0x00}},      {"blue", 4, {0x00, 0x00, 0x80}},
  {"brblack", 8, {0x80, 0x80, 0x80}},    {"brblue", 12, {0x00, 0x00, 0xFF}},
  {"brbrown", 11, {0xFF, 0xFF, 0x00}},    {"brcyan", 14, {0x00, 0xFF, 0xFF}},
  {"brgreen", 10, {0x00, 0xFF, 0x00}},   {"brgrey", 8, {0x55, 0x55, 0x55}},
  {"brmagenta", 13, {0xFF, 0x00, 0xFF}}, {"brown", 3, {0x72, 0x50, 0x00}},
  {"brpurple", 13, {0xFF, 0x00, 0xFF}},   {"brred", 9, {0xFF, 0x00, 0x00}},
  {"brwhite", 15, {0xFF, 0xFF, 0xFF}},   {"bryellow", 11, {0xFF, 0xFF, 0x00}},
  {"cyan", 6, {0x00, 0x80, 0x80}},       {"green", 2, {0x00, 0x80, 0x00}},
  {"grey", 7, {0xE5, 0xE5, 0xE5}},        {"magenta", 5, {0x80, 0x00, 0x80}},
  {"purple", 5, {0x80, 0x00, 0x80}},      {"red", 1, {0x80, 0x00, 0x00}},
  {"white", 7, {0xC0, 0xC0, 0xC0}},      {"yellow", 3, {0x80, 0x80, 0x00}},
};
static int num_named_colors = sizeof(named_colors) / sizeof(named_colors[0]);

int find_named_color(int l, int r, const char* x) {
  if (r >= l) {
    int mid = l + (r - l) / 2;

    if (simple_icase_compare(named_colors[mid].name, x) == 0)
      return mid;

    if (simple_icase_compare(named_colors[mid].name, x) > 0)
      return find_named_color(l, mid - 1, x);

    return find_named_color(mid + 1, r, x);
  }

  return -1;
}

Color* try_parse_named(const char* str) {
  if (str == NULL) {
    return NULL;
  }

  int idx = find_named_color(0, num_named_colors, str);
  if (idx != -1 && simple_icase_compare(named_colors[idx].name, str) == 0) {
    Color* color = malloc(sizeof(Color));
    color->type = COLOR_TYPE_NAMED;
    color->name_idx = named_colors[idx].idx;
    return color;
  }
  return NULL;
}

static uint8_t term16_color_for_rgb(const uint8_t rgb[3]) {
  const uint32_t kColors[] = {
    0x000000,  // Black
    0x800000,  // Red
    0x008000,  // Green
    0x808000,  // Yellow
    0x000080,  // Blue
    0x800080,  // Magenta
    0x008080,  // Cyan
    0xc0c0c0,  // White
    0x808080,  // Bright Black
    0xFF0000,  // Bright Red
    0x00FF00,  // Bright Green
    0xFFFF00,  // Bright Yellow
    0x0000FF,  // Bright Blue
    0xFF00FF,  // Bright Magenta
    0x00FFFF,  // Bright Cyan
    0xFFFFFF   // Bright White
  };
  return convert_color(rgb, kColors, sizeof kColors / sizeof *kColors);
}

static uint8_t term256_color_for_rgb(const uint8_t rgb[3]) {
  const uint32_t kColors[240] = {
    0x000000, 0x00005f, 0x000087, 0x0000af, 0x0000d7, 0x0000ff, 0x005f00, 0x005f5f, 0x005f87,
    0x005faf, 0x005fd7, 0x005fff, 0x008700, 0x00875f, 0x008787, 0x0087af, 0x0087d7, 0x0087ff,
    0x00af00, 0x00af5f, 0x00af87, 0x00afaf, 0x00afd7, 0x00afff, 0x00d700, 0x00d75f, 0x00d787,
    0x00d7af, 0x00d7d7, 0x00d7ff, 0x00ff00, 0x00ff5f, 0x00ff87, 0x00ffaf, 0x00ffd7, 0x00ffff,
    0x5f0000, 0x5f005f, 0x5f0087, 0x5f00af, 0x5f00d7, 0x5f00ff, 0x5f5f00, 0x5f5f5f, 0x5f5f87,
    0x5f5faf, 0x5f5fd7, 0x5f5fff, 0x5f8700, 0x5f875f, 0x5f8787, 0x5f87af, 0x5f87d7, 0x5f87ff,
    0x5faf00, 0x5faf5f, 0x5faf87, 0x5fafaf, 0x5fafd7, 0x5fafff, 0x5fd700, 0x5fd75f, 0x5fd787,
    0x5fd7af, 0x5fd7d7, 0x5fd7ff, 0x5fff00, 0x5fff5f, 0x5fff87, 0x5fffaf, 0x5fffd7, 0x5fffff,
    0x870000, 0x87005f, 0x870087, 0x8700af, 0x8700d7, 0x8700ff, 0x875f00, 0x875f5f, 0x875f87,
    0x875faf, 0x875fd7, 0x875fff, 0x878700, 0x87875f, 0x878787, 0x8787af, 0x8787d7, 0x8787ff,
    0x87af00, 0x87af5f, 0x87af87, 0x87afaf, 0x87afd7, 0x87afff, 0x87d700, 0x87d75f, 0x87d787,
    0x87d7af, 0x87d7d7, 0x87d7ff, 0x87ff00, 0x87ff5f, 0x87ff87, 0x87ffaf, 0x87ffd7, 0x87ffff,
    0xaf0000, 0xaf005f, 0xaf0087, 0xaf00af, 0xaf00d7, 0xaf00ff, 0xaf5f00, 0xaf5f5f, 0xaf5f87,
    0xaf5faf, 0xaf5fd7, 0xaf5fff, 0xaf8700, 0xaf875f, 0xaf8787, 0xaf87af, 0xaf87d7, 0xaf87ff,
    0xafaf00, 0xafaf5f, 0xafaf87, 0xafafaf, 0xafafd7, 0xafafff, 0xafd700, 0xafd75f, 0xafd787,
    0xafd7af, 0xafd7d7, 0xafd7ff, 0xafff00, 0xafff5f, 0xafff87, 0xafffaf, 0xafffd7, 0xafffff,
    0xd70000, 0xd7005f, 0xd70087, 0xd700af, 0xd700d7, 0xd700ff, 0xd75f00, 0xd75f5f, 0xd75f87,
    0xd75faf, 0xd75fd7, 0xd75fff, 0xd78700, 0xd7875f, 0xd78787, 0xd787af, 0xd787d7, 0xd787ff,
    0xd7af00, 0xd7af5f, 0xd7af87, 0xd7afaf, 0xd7afd7, 0xd7afff, 0xd7d700, 0xd7d75f, 0xd7d787,
    0xd7d7af, 0xd7d7d7, 0xd7d7ff, 0xd7ff00, 0xd7ff5f, 0xd7ff87, 0xd7ffaf, 0xd7ffd7, 0xd7ffff,
    0xff0000, 0xff005f, 0xff0087, 0xff00af, 0xff00d7, 0xff00ff, 0xff5f00, 0xff5f5f, 0xff5f87,
    0xff5faf, 0xff5fd7, 0xff5fff, 0xff8700, 0xff875f, 0xff8787, 0xff87af, 0xff87d7, 0xff87ff,
    0xffaf00, 0xffaf5f, 0xffaf87, 0xffafaf, 0xffafd7, 0xffafff, 0xffd700, 0xffd75f, 0xffd787,
    0xffd7af, 0xffd7d7, 0xffd7ff, 0xffff00, 0xffff5f, 0xffff87, 0xffffaf, 0xffffd7, 0xffffff,
    0x080808, 0x121212, 0x1c1c1c, 0x262626, 0x303030, 0x3a3a3a, 0x444444, 0x4e4e4e, 0x585858,
    0x626262, 0x6c6c6c, 0x767676, 0x808080, 0x8a8a8a, 0x949494, 0x9e9e9e, 0xa8a8a8, 0xb2b2b2,
    0xbcbcbc, 0xc6c6c6, 0xd0d0d0, 0xdadada, 0xe4e4e4, 0xeeeeee};
  return 16 + convert_color(rgb, kColors, sizeof kColors / sizeof *kColors);
}

bool color_idx_matches_vterm_color(unsigned char idx, VTermColor* vc) {
  if (!VTERM_COLOR_IS_INDEXED(vc)) {
    return false;
  }
  uint8_t index = vc->indexed.idx;
  if (idx >= 16) {
    return index == idx;
  }

  return index == idx;
}

// See fish's output.cpp:parse_color
Color* parse_color_from_string(const char* str, color_support_t color_support) {
  const char* delims = " \t";
  Color* first_rgb = NULL;
  Color* first_named = NULL;

  log_info("Parsing fish color for string: %s", str);
  char* tmp = strdup(str);
  char* color_name = strtok(tmp, delims);
  while (color_name != NULL) {
    if (color_name[0] != '-') {
      Color* color = try_parse_named(color_name);
      if (color == NULL) color = try_parse_rgb(color_name);
      if (color != NULL) {
        if (first_rgb == NULL && color->type == COLOR_TYPE_RGB) {
          first_rgb = color;
        } else if (first_named == NULL && color->type == COLOR_TYPE_NAMED) {
          first_named = color;
        } else {
          free(color);
        }
      }
    }
    color_name = strtok(NULL, delims);
  }
  free(tmp);

  if ((first_rgb != NULL && color_support & color_support_term256) || first_named == NULL) {
    free(first_named);
    return first_rgb;
  }
  free(first_rgb);
  return first_named;
}

VTermColor* color_to_vterm_color(Color* c, color_support_t color_support) {
  if (c == NULL) {
    return NULL;
  }
  VTermColor* vc = malloc(sizeof(VTermColor));
  if (c->type == COLOR_TYPE_RGB) {
    if (color_support & color_support_term24bit) {
      vterm_color_rgb(vc, c->rgb[0], c->rgb[1], c->rgb[2]);
    } else if (color_support & color_support_term256) {
      vterm_color_indexed(vc, term256_color_for_rgb(c->rgb));
    } else {
      vterm_color_indexed(vc, term16_color_for_rgb(c->rgb));
    }
  } else {
    // TODO(sean): fish will do idx -= 8 if only 8 colors are supported, but we
    // don't know exactly how many colors are supported without curses.
    vterm_color_indexed(vc, c->name_idx);
  }
  return vc;
}

VTermColor* parse_vterm_color_from_string(const char* str, color_support_t color_support) {
  Color* c = parse_color_from_string(str, color_support);
  VTermColor* vc = color_to_vterm_color(c, color_support);
  free(c);
  return vc;
}
