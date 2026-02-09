
package org.globalart.astrum.repository.content.tasks.normalize;

import org.globalart.astrum.common.event.EventWithSource;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Astrum Event to indicate if the NormalizeComponentVersionTask has finished for the specified format
 */
public class FormatVersionNormalizedEvent
    extends EventWithSource
{
  private String format;

  public FormatVersionNormalizedEvent() {
    // deserialization
  }

  public FormatVersionNormalizedEvent(final String format) {
    this.format = checkNotNull(format);
  }

  public String getFormat() {
    return format;
  }
}
