#include <stdbool.h>
#include <stdint.h>

/**
 * Tagged union storing either an RGB color or an index into a colour palette.
 * In order to convert indexed colours to RGB, you may use the
 * vterm_state_convert_color_to_rgb() or vterm_screen_convert_color_to_rgb()
 * functions which lookup the RGB colour from the palette maintained by a
 * VTermState or VTermScreen instance.
 */
typedef union {
  /**
   * Tag indicating which union member is actually valid. This variable
   * coincides with the `type` member of the `rgb` and the `indexed` struct
   * in memory. Please use the `VTERM_COLOR_IS_*` test macros to check whether
   * a particular type flag is set.
   */
  uint8_t type;

  /**
   * Valid if `VTERM_COLOR_IS_RGB(type)` is true. Holds the RGB colour values.
   */
  struct {
    /**
     * Same as the top-level `type` member stored in VTermColor.
     */
    uint8_t type;

    /**
     * The actual 8-bit red, green, blue colour values.
     */
    uint8_t red, green, blue;
  } rgb;

  /**
   * If `VTERM_COLOR_IS_INDEXED(type)` is true, this member holds the index into
   * the colour palette.
   */
  struct {
    /**
     * Same as the top-level `type` member stored in VTermColor.
     */
    uint8_t type;

    /**
     * Index into the colour map.
     */
    uint8_t idx;
  } indexed;
} VTermColor;

/**
 * Bit-field describing the content of the tagged union `VTermColor`.
 */
typedef enum {
  /**
   * If the lower bit of `type` is not set, the colour is 24-bit RGB.
   */
  VTERM_COLOR_RGB = 0x00,

  /**
   * The colour is an index into a palette of 256 colours.
   */
  VTERM_COLOR_INDEXED = 0x01,

  /**
   * Mask that can be used to extract the RGB/Indexed bit.
   */
  VTERM_COLOR_TYPE_MASK = 0x01,

  /**
   * If set, indicates that this colour should be the default foreground
   * color, i.e. there was no SGR request for another colour. When
   * rendering this colour it is possible to ignore "idx" and just use a
   * colour that is not in the palette.
   */
  VTERM_COLOR_DEFAULT_FG = 0x02,

  /**
   * If set, indicates that this colour should be the default background
   * color, i.e. there was no SGR request for another colour. A common
   * option when rendering this colour is to not render a background at
   * all, for example by rendering the window transparently at this spot.
   */
  VTERM_COLOR_DEFAULT_BG = 0x04,

  /**
   * Mask that can be used to extract the default foreground/background bit.
   */
  VTERM_COLOR_DEFAULT_MASK = 0x06
} VTermColorType;

/**
 * Returns true if the VTERM_COLOR_RGB `type` flag is set, indicating that the
 * given VTermColor instance is an indexed colour.
 */
#define VTERM_COLOR_IS_INDEXED(col) \
  (((col)->type & VTERM_COLOR_TYPE_MASK) == VTERM_COLOR_INDEXED)

bool vterm_color_is_indexed(const VTermColor *col) {
  return VTERM_COLOR_IS_INDEXED(col);
}

/**
 * Returns true if the VTERM_COLOR_INDEXED `type` flag is set, indicating that
 * the given VTermColor instance is an rgb colour.
 */
#define VTERM_COLOR_IS_RGB(col) \
  (((col)->type & VTERM_COLOR_TYPE_MASK) == VTERM_COLOR_RGB)

bool vterm_color_is_rgb(const VTermColor *col) {
  return VTERM_COLOR_IS_RGB(col);
}

/**
 * Construct a new VTermColor instance representing an indexed color with the
 * given index.
 */
static inline void vterm_color_indexed(VTermColor *col, uint8_t idx)
{
  col->type = VTERM_COLOR_INDEXED;
  col->indexed.idx = idx;
}

/**
 * Constructs a new VTermColor instance representing the given RGB values.
 */
static inline void vterm_color_rgb(VTermColor *col, uint8_t red, uint8_t green,
                                   uint8_t blue)
{
  col->type = VTERM_COLOR_RGB;
  col->rgb.red   = red;
  col->rgb.green = green;
  col->rgb.blue  = blue;
}

enum { color_support_term256 = 1 << 0, color_support_term24bit = 1 << 1 };
typedef unsigned int color_support_t;
color_support_t get_color_support();

typedef struct {
  VTermColor* fg;
  VTermColor* bg;
} SuggestionColor;

SuggestionColor* parse_suggestion_color_fish(const char*, color_support_t);
SuggestionColor* parse_suggestion_color_zsh_autosuggest(const char*, color_support_t);
void free_suggestion_color(SuggestionColor*);
