package org.globalart.astrum.repository.huggingface.internal;

import java.util.Arrays;
import java.util.Optional;

public enum RepoType
{
  MODEL("model", "models", ""),
  DATASET("dataset", "datasets", "datasets/"),
  SPACE("space", "spaces", "spaces/");

  private final String kind;

  private final String plural;

  private final String urlPrefix;

  RepoType(final String kind, final String plural, final String urlPrefix) {
    this.kind = kind;
    this.plural = plural;
    this.urlPrefix = urlPrefix;
  }

  public String kind() {
    return kind;
  }

  public String plural() {
    return plural;
  }

  public String urlPrefix() {
    return urlPrefix;
  }

  public static Optional<RepoType> fromPlural(final String value) {
    return Arrays.stream(values()).filter(type -> type.plural.equals(value)).findFirst();
  }
}
