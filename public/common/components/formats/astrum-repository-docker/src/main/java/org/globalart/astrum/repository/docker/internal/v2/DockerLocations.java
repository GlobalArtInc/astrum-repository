package org.globalart.astrum.repository.docker.internal.v2;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.repository.docker.internal.security.DockerAuth;
import org.globalart.astrum.repository.view.Context;

public final class DockerLocations
{
  private DockerLocations() {
  }

  public static String prefix(final Context context) {
    Object prefix = context.getRequest().getAttributes().get(DockerAuth.PREFIX_ATTRIBUTE);
    if (prefix != null) {
      return prefix.toString();
    }
    String base = BaseUrlHolder.isSet() ? BaseUrlHolder.get() : "";
    return base + "/repository/" + context.getRepository().getName() + "/v2/";
  }

  public static String externalName(final Context context, final String name) {
    Object namePrefix = context.getRequest().getAttributes().get(DockerAuth.NAME_PREFIX_ATTRIBUTE);
    return namePrefix == null ? name : namePrefix + name;
  }

  public static String blob(final Context context, final String name, final String digest) {
    return prefix(context) + name + "/blobs/" + digest;
  }

  public static String manifest(final Context context, final String name, final String reference) {
    return prefix(context) + name + "/manifests/" + reference;
  }

  public static String upload(final Context context, final String name, final String uuid) {
    return prefix(context) + name + "/blobs/uploads/" + uuid;
  }

  public static String referrers(final Context context, final String name, final String digest) {
    return prefix(context) + name + "/referrers/" + digest;
  }

  public static String tags(final Context context, final String name) {
    return prefix(context) + name + "/tags/list";
  }

  public static String catalog(final Context context) {
    return prefix(context) + "_catalog";
  }

  public static String scope(final Context context, final String name, final boolean push) {
    return "repository:" + externalName(context, name) + (push ? ":pull,push" : ":pull");
  }
}
