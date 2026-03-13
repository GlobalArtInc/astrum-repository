
package org.globalart.astrum.repository.search.sql.index;

import java.util.regex.Pattern;

public class SearchTokenizer
{
  public static final Pattern TOKENIZER = Pattern.compile("[_.\\-/\\\\ ,]");

  private static final String QUOTE = "'";

  public static String tsEscape(final String term) {
    return QUOTE + term.toLowerCase().replace("\\", "\\\\").replace(QUOTE, "\\'") + QUOTE;
  }

  /**
   * Strips leading token separators from the provided term string
   */
  public static String stripLeadingSeparators(final String term) {
    int index = -1;

    for (int i = 0; i < term.length(); i++) {
      if (!isSeparator(term.charAt(i))) {
        break;
      }
      index = i;
    }

    return index > -1 ? term.substring(index + 1) : term;
  }

  /**
   * Strips trailing token separators from the provided term string
   */
  public static String stripTrailingSeparators(final String term) {
    int index = term.length();

    for (int i = term.length() - 1; i >= 0; i--) {
      if (!isSeparator(term.charAt(i))) {
        break;
      }
      index = i;
    }

    return index < term.length() ? term.substring(0, index) : term;
  }

  public static boolean isSeparator(final char c) {
    return c == ' ' || c == '-' || c == '\\' || c == '/' || c == '.' || c == '_' || c == ',';
  }
}
