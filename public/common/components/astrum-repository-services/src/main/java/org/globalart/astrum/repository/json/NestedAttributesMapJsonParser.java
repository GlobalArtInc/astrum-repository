
package org.globalart.astrum.repository.json;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;

import com.fasterxml.jackson.core.JsonParser;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Objects.isNull;

/**
 * {@link JsonParser} that holds state for communicating with a {@link NestedAttributesMap}. This mostly will be
 * valuable for parsing and merging multiple json objects into a give {@link NestedAttributesMap}
 *
 * @since 3.16
 */
public class NestedAttributesMapJsonParser
    extends CurrentPathJsonParser
{
  private final NestedAttributesMap root;

  private boolean mappingInsideArray;

  private boolean defaultMapping;

  public NestedAttributesMapJsonParser(final JsonParser jsonParser, final NestedAttributesMap root) {
    super(checkNotNull(jsonParser));
    this.root = checkNotNull(root);
  }

  /**
   * Attempt to retrieve the {@link NestedAttributesMap} that is associated with
   * the current path, if parts of the path are not existing they will
   * be manually created by the {@link NestedAttributesMap#child(String)} for that path part.
   *
   * @return NestedAttributesMap or null if we are mapping inside an array or no child was found.
   */
  @Nullable
  public NestedAttributesMap getChildFromRoot() {
    // if we are inside an array we don't have a map value to return
    if (isMappingInsideArray()) {
      return null;
    }

    // we start with the root as initial existing child to go down it's path
    NestedAttributesMap existingChild = root;

    // remove the first "/" and then split on any leftover
    for (String part : currentPathInParts()) {
      Object o = existingChild.get(part);

      // only create and/or retrieve other child json objects,
      if (isNull(o) || o instanceof Map) {
        existingChild = existingChild.child(part);
      }
    }

    return existingChild.equals(root) ? null : existingChild;
  }

  /**
   * Inform parser that the current position is inside an array.
   */
  public void markMappingInsideArray() {
    mappingInsideArray = true;
  }

  /**
   * Inform parser that the current position has existed an array.
   */
  public void unMarkMappingInsideArray() {
    mappingInsideArray = false;
  }

  /**
   * Inform parser that no special mapping should occur but the default Jackson mapping should be followed.
   */
  public void enableDefaultMapping() {
    defaultMapping = true;
  }

  /**
   * Inform parser that special mapping should occur over using the default Jackson mapping.
   */
  public void disableDefaultMapping() {
    defaultMapping = false;
  }

  public NestedAttributesMap getRoot() {
    return root;
  }

  public boolean isMappingInsideArray() {
    return mappingInsideArray;
  }

  public boolean isDefaultMapping() {
    return defaultMapping;
  }
}
