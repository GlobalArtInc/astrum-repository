
package org.globalart.astrum.repository.r.internal.util;

import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.TreeMap;

import javax.annotation.Nullable;

import static java.util.Collections.unmodifiableMap;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_MD5SUM;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_PACKAGE;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_VERSION;
import static org.globalart.astrum.repository.r.internal.RAttributes.PACKAGES_FIELDS;
import static org.globalart.astrum.repository.r.internal.util.RMetadataUtils.writeRecord;

public class RPackagesBuilder
{
  private final Map<String, RPackageVersion> packageVersions = new HashMap<>();

  private final Map<String, Map<String, String>> packageInformation = new TreeMap<>();

  public Map<String, Map<String, String>> getPackageInformation() {
    return unmodifiableMap(packageInformation);
  }

  public void append(final Map<String, ?> description, @Nullable final String md5) {
    Object packageName = description.get(P_PACKAGE);
    Object version = description.get(P_VERSION);
    if (packageName == null || version == null) {
      return;
    }
    RPackageVersion oldVersion = packageVersions.get(packageName.toString());
    RPackageVersion newVersion = new RPackageVersion(version.toString());
    if (oldVersion == null || newVersion.compareTo(oldVersion) > 0) {
      Map<String, String> information = new LinkedHashMap<>();
      for (String field : PACKAGES_FIELDS) {
        Object value = description.get(field);
        if (value != null) {
          information.put(field, value.toString());
        }
      }
      if (md5 != null) {
        information.put(P_MD5SUM, md5);
      }
      packageVersions.put(packageName.toString(), newVersion);
      packageInformation.put(packageName.toString(), information);
    }
  }

  public boolean isEmpty() {
    return packageInformation.isEmpty();
  }

  public String buildPackages() {
    StringBuilder packages = new StringBuilder();
    for (Map<String, String> information : packageInformation.values()) {
      if (!packages.isEmpty()) {
        packages.append('\n');
      }
      packages.append(writeRecord(information, PACKAGES_FIELDS));
    }
    return packages.toString();
  }
}
