package org.globalart.astrum.repository.rubygems.internal.gem;

import java.util.List;

public record GemDependency(String name, String type, List<String> requirements)
{
  public static final String RUNTIME = "runtime";

  public static final String DEVELOPMENT = "development";

  public GemDependency {
    requirements = requirements.isEmpty() ? List.of(GemRequirements.ANY) : List.copyOf(requirements);
  }

  public boolean isRuntime() {
    return !DEVELOPMENT.equals(type);
  }
}
