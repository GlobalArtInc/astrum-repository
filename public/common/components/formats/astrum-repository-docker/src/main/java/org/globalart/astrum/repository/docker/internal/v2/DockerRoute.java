package org.globalart.astrum.repository.docker.internal.v2;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Context;

public class DockerRoute
{
  public enum Type
  {
    BASE,
    CATALOG,
    TAGS,
    MANIFEST,
    BLOB,
    UPLOADS,
    UPLOAD,
    REFERRERS
  }

  private final Type type;

  private final String name;

  private final String reference;

  public DockerRoute(final Type type, @Nullable final String name, @Nullable final String reference) {
    this.type = type;
    this.name = name;
    this.reference = reference;
  }

  public Type getType() {
    return type;
  }

  @Nullable
  public String getName() {
    return name;
  }

  @Nullable
  public String getReference() {
    return reference;
  }

  public boolean isDigestReference() {
    return reference != null && DockerDigest.isDigest(reference);
  }

  public static DockerRoute get(final Context context) {
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    if (route == null) {
      throw new IllegalStateException("Docker route not resolved for " + context.getRequest().getPath());
    }
    return route;
  }

  @Override
  public String toString() {
    return type + "{name=" + name + ", reference=" + reference + "}";
  }
}
