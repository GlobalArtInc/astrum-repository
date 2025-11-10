
package org.globalart.astrum.bootstrap.entrypoint.edition;

import java.util.List;

public interface AstrumEdition
{
  String getId();

  String getName();

  String getShortName();

  int getPriority();

  boolean isActive();

  List<String> getModules();
}
