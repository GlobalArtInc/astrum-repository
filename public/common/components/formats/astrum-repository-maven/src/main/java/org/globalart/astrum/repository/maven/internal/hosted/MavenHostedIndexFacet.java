
package org.globalart.astrum.repository.maven.internal.hosted;

import java.io.IOException;

import org.globalart.astrum.repository.maven.MavenIndexFacet;

public interface MavenHostedIndexFacet
    extends MavenIndexFacet
{
  void publishIndex() throws IOException;
}
