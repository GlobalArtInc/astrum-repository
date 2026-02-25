
package org.globalart.astrum.repository.search.sql;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Spring-managed configuration for SQL search wildcard validation.
 * Reads the minimum prefix length from astrum.properties and configures the validator.
 *
 * <p>
 * This class is the single authority for validation of the prefix length configuration.
 * The validated value is then passed to {@link SqlSearchWildcardValidator}.
 */
@Component
public class SqlSearchWildcardConfiguration
{
  private static final Logger log = LoggerFactory.getLogger(SqlSearchWildcardConfiguration.class);

  private static final int DEFAULT_MIN_ALLOWED = 3;

  /**
   * Constructs the configuration by reading the value from astrum.properties
   * and immediately applying it to the validator.
   *
   * <p>
   * This constructor validates the configuration value and applies it to the validator.
   * Values must be between 1 and 100. Invalid values are rejected with a warning log
   * and the default value (3) is used instead.
   *
   * @param minAllowedSymbolsToSearch minimum characters required before a trailing wildcard.
   *          Defaults to 3 if not specified. Spring injects this from astrum.properties.
   */
  public SqlSearchWildcardConfiguration(
      @Value("${astrum.search.wildcard.minPrefixLength:3}") final int minAllowedSymbolsToSearch)
  {
    int value = minAllowedSymbolsToSearch;
    if (value < 1 || value > 100) {
      log.warn("Invalid minimum prefix length: {}. Must be between 1 and 100. Using default: {}",
          value, DEFAULT_MIN_ALLOWED);
      value = DEFAULT_MIN_ALLOWED;
    }
    else if (value != DEFAULT_MIN_ALLOWED) {
      log.info("Wildcard search minimum prefix length set to {} (default is {})",
          value, DEFAULT_MIN_ALLOWED);
    }
    SqlSearchWildcardValidator.setMinAllowedSymbolsToSearch(value);
  }
}
