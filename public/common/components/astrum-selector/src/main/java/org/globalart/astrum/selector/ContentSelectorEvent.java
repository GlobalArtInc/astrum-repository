
package org.globalart.astrum.selector;

import static com.google.common.base.Preconditions.checkNotNull;

public class ContentSelectorEvent
{
  private final SelectorConfiguration selectorConfiguration;

  public ContentSelectorEvent(final SelectorConfiguration selectorConfiguration) {
    this.selectorConfiguration = checkNotNull(selectorConfiguration);
  }

  public SelectorConfiguration getSelectorConfiguration() {
    return selectorConfiguration;
  }
}
