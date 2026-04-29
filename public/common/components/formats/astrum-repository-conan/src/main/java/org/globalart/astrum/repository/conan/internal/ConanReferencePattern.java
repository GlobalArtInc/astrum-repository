package org.globalart.astrum.repository.conan.internal;

import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.conan.ConanCoords;

public final class ConanReferencePattern
{
  private final Pattern pattern;

  private final String namePattern;

  private ConanReferencePattern(@Nullable final String query, final boolean ignoreCase) {
    String text = query == null || query.isBlank() ? "*" : query.trim();
    this.pattern = Pattern.compile(globToRegex(text), ignoreCase ? Pattern.CASE_INSENSITIVE : 0);
    int slash = text.indexOf('/');
    int at = text.indexOf('@');
    int end = slash >= 0 ? slash : at >= 0 ? at : -1;
    this.namePattern = end >= 0 ? text.substring(0, end) : text;
  }

  public static ConanReferencePattern of(@Nullable final String query, final boolean ignoreCase) {
    return new ConanReferencePattern(query, ignoreCase);
  }

  public String namePattern() {
    return namePattern;
  }

  public String nameLikePattern() {
    StringBuilder like = new StringBuilder();
    for (char c : namePattern.toLowerCase().toCharArray()) {
      switch (c) {
        case '*' -> like.append('%');
        case '?' -> like.append('_');
        case '%', '_', '\\' -> like.append('\\').append(c);
        default -> like.append(c);
      }
    }
    return like.toString();
  }

  public boolean matches(final ConanCoords coords) {
    StringBuilder partial = new StringBuilder(coords.getName());
    if (pattern.matcher(partial).matches()) {
      return true;
    }
    partial.append('/').append(coords.getVersion());
    if (pattern.matcher(partial).matches()) {
      return true;
    }
    if (!ConanCoords.NONE.equals(coords.getUser())) {
      partial.append('@').append(coords.getUser());
      if (pattern.matcher(partial).matches()) {
        return true;
      }
      if (!ConanCoords.NONE.equals(coords.getChannel())) {
        partial.append('/').append(coords.getChannel());
        if (pattern.matcher(partial).matches()) {
          return true;
        }
      }
    }
    return pattern.matcher(coords.reference()).matches();
  }

  static String globToRegex(final String glob) {
    StringBuilder regex = new StringBuilder();
    for (char c : glob.toCharArray()) {
      switch (c) {
        case '*' -> regex.append(".*");
        case '?' -> regex.append('.');
        default -> {
          if ("\\.[]{}()+-^$|".indexOf(c) >= 0) {
            regex.append('\\');
          }
          regex.append(c);
        }
      }
    }
    return regex.toString();
  }
}
