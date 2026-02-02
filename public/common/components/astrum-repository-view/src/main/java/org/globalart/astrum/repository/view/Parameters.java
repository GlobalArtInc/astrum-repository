
package org.globalart.astrum.repository.view;

import org.globalart.astrum.common.collect.StringMultimap;

import com.google.common.collect.ListMultimap;

/**
 * Parameters container.
 *
 * @since 3.0
 */
public class Parameters
    extends StringMultimap
{
  public Parameters(final ListMultimap<String, String> entries) {
    super(entries);
  }

  public Parameters() {
    super();
  }
}
