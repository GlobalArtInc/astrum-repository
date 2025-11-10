
package org.globalart.astrum.bootstrap.entrypoint.edition;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;

import org.springframework.stereotype.Component;

@Component
public class AstrumEditionSelector
{
  private final List<AstrumEdition> editions;

  @Autowired
  public AstrumEditionSelector(final List<AstrumEdition> editions) {
    this.editions = new ArrayList<>(editions);
    this.editions.sort(Comparator.comparingInt(AstrumEdition::getPriority));
  }

  public AstrumEdition getCurrent() {
    return editions
        .stream()
        .filter(AstrumEdition::isActive)
        .findFirst()
        .orElseThrow(() -> new IllegalStateException("No active edition found"));
  }
}
