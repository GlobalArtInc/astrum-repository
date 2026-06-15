
package org.globalart.astrum.coreui.internal;

import java.util.Collection;
import java.util.Map;

import org.globalart.astrum.coreui.RepositoryXO;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Sets.newLinkedHashSet;
import static java.util.Objects.isNull;
import static java.util.Objects.nonNull;

/**
 * Repository / Cleanup Util that provide utilities for the the cleanup attributes.
 *
 * @since 3.19
 */
public class RepositoryCleanupAttributesUtil
{
  private static final String CLEANUP_ATTRIBUTES_KEY = "cleanup";

  private static final String CLEANUP_NAME_KEY = "policyName";

  private RepositoryCleanupAttributesUtil() {
  }

  /**
   * Based on a given {@link RepositoryXO} we will assure that we remove the <code>cleanup</code> attribute if it has
   * <code>policyName</code> field but without value. If it does have value we update it to make sure it's
   * a {@link java.util.Set} and not any other type of collections.
   *
   * @param repositoryXO - {@link RepositoryXO}
   */
  public static void initializeCleanupAttributes(final RepositoryXO repositoryXO) {
    checkNotNull(repositoryXO);

    Map<String, Map<String, Object>> attributes = checkNotNull(repositoryXO.getAttributes());
    Map<String, Object> cleanup = attributes.get(CLEANUP_ATTRIBUTES_KEY);
    if (nonNull(cleanup)) {

      @SuppressWarnings("unchecked")
      Collection<String> policyNames = (Collection<String>) cleanup.get(CLEANUP_NAME_KEY);

      if (isNull(policyNames) || policyNames.isEmpty()) {
        attributes.remove(CLEANUP_ATTRIBUTES_KEY);
      }
      else {
        cleanup.put(CLEANUP_NAME_KEY, newLinkedHashSet(policyNames));
      }
    }
  }
}
