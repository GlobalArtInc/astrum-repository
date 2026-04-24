
package org.globalart.astrum.repository.r.internal;

import java.util.List;

public final class RAttributes
{
  public static final String P_PACKAGE = "Package";

  public static final String P_VERSION = "Version";

  public static final String P_PRIORITY = "Priority";

  public static final String P_DEPENDS = "Depends";

  public static final String P_IMPORTS = "Imports";

  public static final String P_LINKINGTO = "LinkingTo";

  public static final String P_SUGGESTS = "Suggests";

  public static final String P_ENHANCES = "Enhances";

  public static final String P_LICENSE = "License";

  public static final String P_LICENSE_IS_FOSS = "License_is_FOSS";

  public static final String P_LICENSE_RESTRICTS_USE = "License_restricts_use";

  public static final String P_OS_TYPE = "OS_type";

  public static final String P_ARCHS = "Archs";

  public static final String P_MD5SUM = "MD5sum";

  public static final String P_BUILT = "Built";

  public static final String P_NEEDS_COMPILATION = "NeedsCompilation";

  public static final List<String> PACKAGES_FIELDS = List.of(
      P_PACKAGE,
      P_VERSION,
      P_PRIORITY,
      P_DEPENDS,
      P_IMPORTS,
      P_LINKINGTO,
      P_SUGGESTS,
      P_ENHANCES,
      P_LICENSE,
      P_LICENSE_IS_FOSS,
      P_LICENSE_RESTRICTS_USE,
      P_OS_TYPE,
      P_ARCHS,
      P_MD5SUM,
      P_BUILT,
      P_NEEDS_COMPILATION);

  private RAttributes() {
  }
}
