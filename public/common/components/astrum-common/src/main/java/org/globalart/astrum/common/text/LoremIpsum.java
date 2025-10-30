
package org.globalart.astrum.common.text;

import java.util.List;

import static java.util.Arrays.asList;

/**
 * "Lorem Ipsum" placeholder-text helpers.
 *
 * @since 3.0
 */
public class LoremIpsum
{
  public static final String RAW_TEXT = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod " +
      "tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation " +
      "ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate " +
      "velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in " +
      "culpa qui officia deserunt mollit anim id est laborum.";

  public static final List<String> WORD_LIST = asList(RAW_TEXT.split("\\s"));

  private LoremIpsum() {
    // private constructor
  }

  public static String words(int count) {
    return String.join(" ", WORD_LIST.subList(0, count));
  }
}
