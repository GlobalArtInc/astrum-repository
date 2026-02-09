
package org.globalart.astrum.repository.content.upgrades;

import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;

import org.springframework.stereotype.Component;

/**
 * Adds indexes to {format}_key_value tables for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class KeyValueIndexesMigrationStep_2_65
    extends KeyValueIndexesMigrationStepSupport
{
  @Autowired
  public KeyValueIndexesMigrationStep_2_65(final List<Format> formats) {
    super(formats.stream()
        .map(Format::getValue)
        .toList());
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.65");
  }
}
