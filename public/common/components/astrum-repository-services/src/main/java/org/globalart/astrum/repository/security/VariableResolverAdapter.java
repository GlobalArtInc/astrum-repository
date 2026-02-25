
package org.globalart.astrum.repository.security;

import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.search.AssetSearchResult;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.selector.VariableSource;

/**
 * Generate a variable source from a context, to be used for content selector evaluation
 * 
 * @since 3.1
 */
public interface VariableResolverAdapter
{
  VariableSource fromRequest(Request request, Repository repository);

  /**
   * Creates a {@link VariableSource} from an ES-indexed asset.
   */
  VariableSource fromSourceLookup(String format, Map<String, Object> asset);

  /**
   * @since 3.8
   */
  VariableSource fromCoordinates(String format, String path, Map<String, String> coordinates);

  /**
   * @since 3.18
   */
  VariableSource fromPath(String path, String format);

  VariableSource fromSearchResult(ComponentSearchResult component, AssetSearchResult asset);
}
