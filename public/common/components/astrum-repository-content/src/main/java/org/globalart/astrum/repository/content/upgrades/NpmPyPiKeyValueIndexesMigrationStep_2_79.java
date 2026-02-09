
package org.globalart.astrum.repository.content.upgrades;

import java.util.List;
import java.util.Optional;

import org.springframework.stereotype.Component;

@Component
public class NpmPyPiKeyValueIndexesMigrationStep_2_79
    extends KeyValueIndexesMigrationStepSupport
{
  public NpmPyPiKeyValueIndexesMigrationStep_2_79() {
    super(List.of("npm", "pypi"));
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.79");
  }
}
