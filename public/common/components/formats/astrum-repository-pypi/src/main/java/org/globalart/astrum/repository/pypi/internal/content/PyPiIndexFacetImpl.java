
package org.globalart.astrum.repository.pypi.internal.content;

import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.pypi.content.PypiContentFacet;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.PyPiIndexFacet;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.normalizeName;
import static org.globalart.astrum.repository.pypi.internal.content.ContentPypiPathUtils.indexPath;

@Component
@Qualifier(PyPiFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PyPiIndexFacetImpl
    extends FacetSupport
    implements PyPiIndexFacet
{
  @Override
  public void deleteRootIndex() {
    content().delete(indexPath());
  }

  @Override
  public void deleteIndex(final String packageName) {
    PypiContentFacet contentFacet = content();
    if (!contentFacet.delete(indexPath(normalizeName(packageName)))) {
      contentFacet.delete(indexPath(packageName));
    }
  }

  private PypiContentFacet content() {
    return getRepository().facet(PypiContentFacet.class);
  }
}
