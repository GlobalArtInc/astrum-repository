package org.globalart.astrum.repository.rubygems.internal.gem;

import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public final class GemRequirements
{
  public static final String ANY = ">= 0";

  private static final Pattern REQUIREMENT = Pattern.compile("\\s*(=|!=|>=|<=|>|<|~>)?\\s*(\\S+)\\s*");

  private GemRequirements() {
  }

  public static String[] split(final String requirement) {
    Matcher matcher = REQUIREMENT.matcher(requirement);
    if (!matcher.matches()) {
      throw new IllegalArgumentException("Invalid requirement: " + requirement);
    }
    String operator = matcher.group(1) == null ? "=" : matcher.group(1);
    return new String[]{operator, matcher.group(2)};
  }

  public static String normalize(final String requirement) {
    String[] parts = split(requirement);
    return parts[0] + " " + parts[1];
  }

  public static boolean isAny(final List<String> requirements) {
    return requirements.isEmpty() || requirements.equals(List.of(ANY));
  }
}
