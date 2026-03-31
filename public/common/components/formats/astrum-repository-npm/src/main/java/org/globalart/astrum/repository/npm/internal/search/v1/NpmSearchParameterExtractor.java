
package org.globalart.astrum.repository.npm.internal.search.v1;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.view.Parameters;

import com.google.common.base.Strings;

/**
 * Extractor object allowing easy extraction of information from a {@code Parameters} object as they pertain to npm
 * V1 search, including substitution of sensible defaults.
 *
 * @since 3.7
 */
@Component
public class NpmSearchParameterExtractor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public String extractText(final Parameters parameters) {
    return Strings.nullToEmpty(parameters.get("text")).trim();
  }

  /**
   * Extracts the size parameter, using a default of 20 when not supplied and 250 as a maximum value (following the
   * npm search V1 documentation as of this writing).
   */
  public int extractSize(final Parameters parameters) {
    int size = 20;
    String sizeParameter = Strings.nullToEmpty(parameters.get("size")).trim();
    if (!sizeParameter.isEmpty()) {
      try {
        size = Integer.parseInt(sizeParameter);
      }
      catch (NumberFormatException e) {
        log.debug("Invalid size encountered in search parameters {}, using default", parameters, e);
      }
      if (size < 1) {
        size = 1;
      }
      else if (size > 250) {
        size = 250;
      }
    }
    return size;
  }

  /**
   * Extracts the {@code from} parameter, assuming that the results returned should start at zero when not otherwise
   * specified.
   */
  public int extractFrom(final Parameters parameters) {
    int from = 0;
    String fromParameter = Strings.nullToEmpty(parameters.get("from")).trim();
    if (!fromParameter.isEmpty()) {
      try {
        from = Integer.parseInt(fromParameter);
      }
      catch (NumberFormatException e) {
        log.debug("Invalid from encountered in search parameters {}, using default", parameters, e);
      }
      if (from < 0) {
        from = 0;
      }
    }
    return from;
  }
}
