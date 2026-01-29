
package org.globalart.astrum.repository;

/**
 * @since 3.22
 */
public abstract class FormatFacetSupport
    extends FacetSupport
    implements FormatFacet
{
  @Override
  public Format getFormat() {
    return getRepository().getFormat();
  }

  @Override
  public String getSubFormat() {
    return null;
  }

  @Override
  public void markSubFormat() {
    // noop
  }
}
