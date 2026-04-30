
package org.globalart.astrum.repository.cocoapods.internal;

import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(CocoapodsFormat.NAME)
public class CocoapodsFormat
    extends Format
{
  public static final String NAME = "cocoapods";

  public static final String PACKAGE_NAME_KEY = "package_name";

  public static final String PACKAGE_VERSION_KEY = "package_version";

  public static final String FILENAME_KEY = "filename";

  public CocoapodsFormat() {
    super(NAME);
  }

  public static String removeInitialSlashFromPath(final String path) {
    return path.startsWith("/") ? path.substring(1) : path;
  }
}
