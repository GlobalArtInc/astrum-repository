
package org.globalart.astrum.rapture.internal;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class BundleStateContributor
    implements StateContributor
{
  public static final String STATE_ID = "activeBundles";

  public static final String ASTRUM_MODULE_PREFIX = "astrum-";

  private List<String> modules;

  private final AstrumEditionSelector astrumEditionSelector;

  @Autowired
  public BundleStateContributor(final AstrumEditionSelector astrumEditionSelector) {
    this.astrumEditionSelector = astrumEditionSelector;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(STATE_ID, getModules());
  }

  private List<String> getModules() {
    if (modules == null) {
      modules = astrumEditionSelector.getCurrent()
          .getModules()
          .stream()
          .filter(BundleStateContributor::isInternalModule)
          .toList();
    }
    return modules;
  }

  private static boolean isInternalModule(final String moduleName) {
    // Filter to include only internal Astrum modules (exclude external dependencies)
    return moduleName.startsWith(ASTRUM_MODULE_PREFIX);
  }
}
